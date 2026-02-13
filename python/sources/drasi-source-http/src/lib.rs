use std::collections::HashMap;
use std::sync::Mutex;

use _drasi_core::builder::source_to_capsule;
use _drasi_core::errors::map_err;
use drasi_source_http::config::{
    AuthConfig, BearerConfig, CorsConfig, ElementType, ErrorBehavior, HttpMethod,
    MappingCondition, OperationType, SignatureAlgorithm, SignatureConfig, SignatureEncoding,
    WebhookConfig, WebhookMapping, WebhookRoute,
};
use drasi_source_http::{HttpSource, HttpSourceBuilder, HttpSourceConfig};
use pyo3::prelude::*;

// ---------------------------------------------------------------------------
// Config wrapper types
// ---------------------------------------------------------------------------

/// A template describing how to map webhook payload data to a graph element.
#[pyclass(name = "ElementTemplate")]
#[derive(Clone)]
pub struct PyElementTemplate {
    pub id: String,
    pub labels: Vec<String>,
    pub properties: Option<serde_json::Value>,
    pub from: Option<String>,
    pub to: Option<String>,
}

#[pymethods]
impl PyElementTemplate {
    /// Create a new ElementTemplate.
    ///
    /// Args:
    ///     id: Expression for the element identifier.
    ///     labels: Labels to assign to the element.
    ///     properties: Optional dict of property mappings.
    ///     from_node: Source node id expression (relations only).
    ///     to_node: Target node id expression (relations only).
    #[new]
    #[pyo3(signature = (id, labels, properties=None, from_node=None, to_node=None))]
    fn new(
        id: String,
        labels: Vec<String>,
        properties: Option<PyObject>,
        from_node: Option<String>,
        to_node: Option<String>,
        py: Python<'_>,
    ) -> PyResult<Self> {
        let props = match properties {
            Some(obj) => {
                let val: serde_json::Value = pythonize::depythonize(&obj.bind(py).clone())
                    .map_err(|e| {
                        pyo3::exceptions::PyValueError::new_err(format!(
                            "Invalid properties: {e}"
                        ))
                    })?;
                Some(val)
            }
            None => None,
        };
        Ok(Self {
            id,
            labels,
            properties: props,
            from: from_node,
            to: to_node,
        })
    }
}

impl PyElementTemplate {
    fn to_rust(&self) -> drasi_source_http::config::ElementTemplate {
        drasi_source_http::config::ElementTemplate {
            id: self.id.clone(),
            labels: self.labels.clone(),
            properties: self.properties.clone(),
            from: self.from.clone(),
            to: self.to.clone(),
        }
    }
}

/// A condition that determines when a webhook mapping should be applied.
#[pyclass(name = "MappingCondition")]
#[derive(Clone)]
pub struct PyMappingCondition {
    pub header: Option<String>,
    pub field: Option<String>,
    pub equals: Option<String>,
    pub contains: Option<String>,
    pub regex: Option<String>,
}

#[pymethods]
impl PyMappingCondition {
    /// Create a new MappingCondition.
    ///
    /// All parameters are keyword-only. Specify a header or field to match
    /// against, with equals, contains, or regex for the comparison.
    #[new]
    #[pyo3(signature = (*, header=None, field=None, equals=None, contains=None, regex=None))]
    fn new(
        header: Option<String>,
        field: Option<String>,
        equals: Option<String>,
        contains: Option<String>,
        regex: Option<String>,
    ) -> Self {
        Self {
            header,
            field,
            equals,
            contains,
            regex,
        }
    }
}

impl PyMappingCondition {
    fn to_rust(&self) -> MappingCondition {
        MappingCondition {
            header: self.header.clone(),
            field: self.field.clone(),
            equals: self.equals.clone(),
            contains: self.contains.clone(),
            regex: self.regex.clone(),
        }
    }
}

fn parse_operation_type(s: &str) -> PyResult<OperationType> {
    match s {
        "insert" => Ok(OperationType::Insert),
        "update" => Ok(OperationType::Update),
        "delete" => Ok(OperationType::Delete),
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Invalid operation: '{s}'. Expected 'insert', 'update', or 'delete'"
        ))),
    }
}

fn parse_element_type(s: &str) -> PyResult<ElementType> {
    match s {
        "node" => Ok(ElementType::Node),
        "relation" => Ok(ElementType::Relation),
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Invalid element_type: '{s}'. Expected 'node' or 'relation'"
        ))),
    }
}

fn parse_http_method(s: &str) -> PyResult<HttpMethod> {
    match s.to_uppercase().as_str() {
        "GET" => Ok(HttpMethod::Get),
        "POST" => Ok(HttpMethod::Post),
        "PUT" => Ok(HttpMethod::Put),
        "PATCH" => Ok(HttpMethod::Patch),
        "DELETE" => Ok(HttpMethod::Delete),
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Invalid HTTP method: '{s}'"
        ))),
    }
}

fn parse_error_behavior(s: &str) -> PyResult<ErrorBehavior> {
    match s {
        "accept_and_log" => Ok(ErrorBehavior::AcceptAndLog),
        "accept_and_skip" => Ok(ErrorBehavior::AcceptAndSkip),
        "reject" => Ok(ErrorBehavior::Reject),
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Invalid error_behavior: '{s}'. Expected 'accept_and_log', 'accept_and_skip', or 'reject'"
        ))),
    }
}

/// Maps an incoming webhook payload to a graph operation (insert, update, or delete).
#[pyclass(name = "WebhookMapping")]
#[derive(Clone)]
pub struct PyWebhookMapping {
    pub when: Option<PyMappingCondition>,
    pub operation: Option<String>,
    pub operation_from: Option<String>,
    pub operation_map: Option<HashMap<String, String>>,
    pub element_type: String,
    pub template: PyElementTemplate,
}

#[pymethods]
impl PyWebhookMapping {
    /// Create a new WebhookMapping.
    ///
    /// Args:
    ///     element_type: 'node' or 'relation'.
    ///     template: ElementTemplate describing the graph element.
    ///     when: Optional condition for this mapping to apply.
    ///     operation: Fixed operation type ('insert', 'update', 'delete').
    ///     operation_from: Field name to read the operation from.
    ///     operation_map: Dict mapping field values to operation types.
    #[new]
    #[pyo3(signature = (element_type, template, *, when=None, operation=None, operation_from=None, operation_map=None))]
    fn new(
        element_type: String,
        template: PyElementTemplate,
        when: Option<PyMappingCondition>,
        operation: Option<String>,
        operation_from: Option<String>,
        operation_map: Option<HashMap<String, String>>,
    ) -> PyResult<Self> {
        // Validate enum values eagerly
        parse_element_type(&element_type)?;
        if let Some(ref op) = operation {
            parse_operation_type(op)?;
        }
        if let Some(ref map) = operation_map {
            for v in map.values() {
                parse_operation_type(v)?;
            }
        }
        Ok(Self {
            when,
            operation,
            operation_from,
            operation_map,
            element_type,
            template,
        })
    }
}

impl PyWebhookMapping {
    fn to_rust(&self) -> PyResult<WebhookMapping> {
        Ok(WebhookMapping {
            when: self.when.as_ref().map(|c| c.to_rust()),
            operation: self
                .operation
                .as_ref()
                .map(|s| parse_operation_type(s))
                .transpose()?,
            operation_from: self.operation_from.clone(),
            operation_map: self
                .operation_map
                .as_ref()
                .map(|m| {
                    m.iter()
                        .map(|(k, v)| parse_operation_type(v).map(|op| (k.clone(), op)))
                        .collect::<PyResult<HashMap<String, OperationType>>>()
                })
                .transpose()?,
            element_type: parse_element_type(&self.element_type)?,
            effective_from: None,
            template: self.template.to_rust(),
        })
    }
}

/// HMAC signature verification configuration for webhook authentication.
#[pyclass(name = "SignatureConfig")]
#[derive(Clone)]
pub struct PySignatureConfig {
    pub algorithm: String,
    pub secret_env: String,
    pub header: String,
    pub prefix: Option<String>,
    pub encoding: Option<String>,
}

#[pymethods]
impl PySignatureConfig {
    /// Create a new SignatureConfig.
    ///
    /// Args:
    ///     algorithm: 'hmac-sha1' or 'hmac-sha256'.
    ///     secret_env: Environment variable holding the shared secret.
    ///     header: HTTP header containing the signature.
    ///     prefix: Optional prefix to strip from the header value.
    ///     encoding: Signature encoding ('hex' or 'base64', default 'hex').
    #[new]
    #[pyo3(signature = (algorithm, secret_env, header, *, prefix=None, encoding=None))]
    fn new(
        algorithm: String,
        secret_env: String,
        header: String,
        prefix: Option<String>,
        encoding: Option<String>,
    ) -> Self {
        Self {
            algorithm,
            secret_env,
            header,
            prefix,
            encoding,
        }
    }
}

impl PySignatureConfig {
    fn to_rust(&self) -> PyResult<SignatureConfig> {
        let algorithm = match self.algorithm.as_str() {
            "hmac-sha1" => SignatureAlgorithm::HmacSha1,
            "hmac-sha256" => SignatureAlgorithm::HmacSha256,
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Invalid algorithm: '{}'. Expected 'hmac-sha1' or 'hmac-sha256'",
                    self.algorithm
                )))
            }
        };
        let encoding = match self.encoding.as_deref() {
            Some("hex") | None => SignatureEncoding::Hex,
            Some("base64") => SignatureEncoding::Base64,
            Some(other) => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Invalid encoding: '{other}'. Expected 'hex' or 'base64'"
                )))
            }
        };
        Ok(SignatureConfig {
            algorithm,
            secret_env: self.secret_env.clone(),
            header: self.header.clone(),
            prefix: self.prefix.clone(),
            encoding,
        })
    }
}

/// Bearer token authentication configuration for webhook routes.
#[pyclass(name = "BearerConfig")]
#[derive(Clone)]
pub struct PyBearerConfig {
    pub token_env: String,
}

#[pymethods]
impl PyBearerConfig {
    /// Create a new BearerConfig.
    ///
    /// Args:
    ///     token_env: Environment variable holding the expected bearer token.
    #[new]
    fn new(token_env: String) -> Self {
        Self { token_env }
    }
}

/// Authentication configuration combining signature and/or bearer strategies.
#[pyclass(name = "AuthConfig")]
#[derive(Clone)]
pub struct PyAuthConfig {
    pub signature: Option<PySignatureConfig>,
    pub bearer: Option<PyBearerConfig>,
}

#[pymethods]
impl PyAuthConfig {
    /// Create a new AuthConfig with optional signature and/or bearer auth.
    #[new]
    #[pyo3(signature = (*, signature=None, bearer=None))]
    fn new(signature: Option<PySignatureConfig>, bearer: Option<PyBearerConfig>) -> Self {
        Self { signature, bearer }
    }
}

impl PyAuthConfig {
    fn to_rust(&self) -> PyResult<AuthConfig> {
        Ok(AuthConfig {
            signature: self
                .signature
                .as_ref()
                .map(|s| s.to_rust())
                .transpose()?,
            bearer: self.bearer.as_ref().map(|b| BearerConfig {
                token_env: b.token_env.clone(),
            }),
        })
    }
}

/// A webhook route that maps incoming HTTP requests to graph operations.
#[pyclass(name = "WebhookRoute")]
#[derive(Clone)]
pub struct PyWebhookRoute {
    pub path: String,
    pub methods: Vec<String>,
    pub auth: Option<PyAuthConfig>,
    pub error_behavior: Option<String>,
    pub mappings: Vec<PyWebhookMapping>,
}

#[pymethods]
impl PyWebhookRoute {
    /// Create a new WebhookRoute.
    ///
    /// Args:
    ///     path: URL path for this route.
    ///     mappings: List of WebhookMapping objects.
    ///     methods: Accepted HTTP methods (default ``["POST"]``).
    ///     auth: Optional authentication configuration.
    ///     error_behavior: 'accept_and_log', 'accept_and_skip', or 'reject'.
    #[new]
    #[pyo3(signature = (path, mappings, *, methods=None, auth=None, error_behavior=None))]
    fn new(
        path: String,
        mappings: Vec<PyWebhookMapping>,
        methods: Option<Vec<String>>,
        auth: Option<PyAuthConfig>,
        error_behavior: Option<String>,
    ) -> PyResult<Self> {
        let methods = methods.unwrap_or_else(|| vec!["POST".to_string()]);
        // Validate methods eagerly
        for m in &methods {
            parse_http_method(m)?;
        }
        if let Some(ref eb) = error_behavior {
            parse_error_behavior(eb)?;
        }
        Ok(Self {
            path,
            methods,
            auth,
            error_behavior,
            mappings,
        })
    }
}

impl PyWebhookRoute {
    fn to_rust(&self) -> PyResult<WebhookRoute> {
        Ok(WebhookRoute {
            path: self.path.clone(),
            methods: self
                .methods
                .iter()
                .map(|m| parse_http_method(m))
                .collect::<PyResult<Vec<HttpMethod>>>()?,
            auth: self.auth.as_ref().map(|a| a.to_rust()).transpose()?,
            error_behavior: self
                .error_behavior
                .as_ref()
                .map(|s| parse_error_behavior(s))
                .transpose()?,
            mappings: self
                .mappings
                .iter()
                .map(|m| m.to_rust())
                .collect::<PyResult<Vec<WebhookMapping>>>()?,
        })
    }
}

/// CORS configuration for the HTTP source webhook server.
#[pyclass(name = "CorsConfig")]
#[derive(Clone)]
pub struct PyCorsConfig {
    pub enabled: bool,
    pub allow_origins: Vec<String>,
    pub allow_methods: Option<Vec<String>>,
    pub allow_headers: Option<Vec<String>>,
    pub expose_headers: Option<Vec<String>>,
    pub allow_credentials: Option<bool>,
    pub max_age: Option<u64>,
}

#[pymethods]
impl PyCorsConfig {
    /// Create a new CorsConfig with keyword-only parameters.
    #[new]
    #[pyo3(signature = (*, enabled=true, allow_origins=None, allow_methods=None, allow_headers=None, expose_headers=None, allow_credentials=None, max_age=None))]
    fn new(
        enabled: bool,
        allow_origins: Option<Vec<String>>,
        allow_methods: Option<Vec<String>>,
        allow_headers: Option<Vec<String>>,
        expose_headers: Option<Vec<String>>,
        allow_credentials: Option<bool>,
        max_age: Option<u64>,
    ) -> Self {
        Self {
            enabled,
            allow_origins: allow_origins.unwrap_or_else(|| vec!["*".to_string()]),
            allow_methods,
            allow_headers,
            expose_headers,
            allow_credentials,
            max_age,
        }
    }
}

impl PyCorsConfig {
    fn to_rust(&self) -> CorsConfig {
        let defaults = CorsConfig::default();
        CorsConfig {
            enabled: self.enabled,
            allow_origins: self.allow_origins.clone(),
            allow_methods: self
                .allow_methods
                .clone()
                .unwrap_or(defaults.allow_methods),
            allow_headers: self
                .allow_headers
                .clone()
                .unwrap_or(defaults.allow_headers),
            expose_headers: self
                .expose_headers
                .clone()
                .unwrap_or(defaults.expose_headers),
            allow_credentials: self.allow_credentials.unwrap_or(defaults.allow_credentials),
            max_age: self.max_age.unwrap_or(defaults.max_age),
        }
    }
}

/// Top-level webhook configuration containing routes and global settings.
#[pyclass(name = "WebhookConfig")]
#[derive(Clone)]
pub struct PyWebhookConfig {
    pub error_behavior: Option<String>,
    pub cors: Option<PyCorsConfig>,
    pub routes: Vec<PyWebhookRoute>,
}

#[pymethods]
impl PyWebhookConfig {
    /// Create a new WebhookConfig.
    ///
    /// Args:
    ///     routes: List of WebhookRoute objects.
    ///     error_behavior: Default error behavior for all routes.
    ///     cors: Optional CORS configuration.
    #[new]
    #[pyo3(signature = (routes, *, error_behavior=None, cors=None))]
    fn new(
        routes: Vec<PyWebhookRoute>,
        error_behavior: Option<String>,
        cors: Option<PyCorsConfig>,
    ) -> PyResult<Self> {
        if let Some(ref eb) = error_behavior {
            parse_error_behavior(eb)?;
        }
        Ok(Self {
            error_behavior,
            cors,
            routes,
        })
    }
}

impl PyWebhookConfig {
    fn to_rust(&self) -> PyResult<WebhookConfig> {
        Ok(WebhookConfig {
            error_behavior: self
                .error_behavior
                .as_ref()
                .map(|s| parse_error_behavior(s))
                .transpose()?
                .unwrap_or_default(),
            cors: self.cors.as_ref().map(|c| c.to_rust()),
            routes: self
                .routes
                .iter()
                .map(|r| r.to_rust())
                .collect::<PyResult<Vec<WebhookRoute>>>()?,
        })
    }
}

/// Typed configuration object for an ``HttpSource``.
#[pyclass(name = "HttpSourceConfig")]
#[derive(Clone)]
pub struct PyHttpSourceConfig {
    pub host: String,
    pub port: u16,
    pub endpoint: Option<String>,
    pub timeout_ms: Option<u64>,
    pub webhooks: Option<PyWebhookConfig>,
}

#[pymethods]
impl PyHttpSourceConfig {
    /// Create a new HttpSourceConfig.
    ///
    /// Args:
    ///     host: HTTP server bind address.
    ///     port: HTTP server port.
    ///     endpoint: Optional base endpoint path.
    ///     timeout_ms: Optional request timeout in milliseconds.
    ///     webhooks: Optional webhook configuration.
    #[new]
    #[pyo3(signature = (host, port, *, endpoint=None, timeout_ms=None, webhooks=None))]
    fn new(
        host: String,
        port: u16,
        endpoint: Option<String>,
        timeout_ms: Option<u64>,
        webhooks: Option<PyWebhookConfig>,
    ) -> Self {
        Self {
            host,
            port,
            endpoint,
            timeout_ms,
            webhooks,
        }
    }
}

impl PyHttpSourceConfig {
    fn to_rust(&self) -> PyResult<HttpSourceConfig> {
        Ok(HttpSourceConfig {
            host: self.host.clone(),
            port: self.port,
            endpoint: self.endpoint.clone(),
            timeout_ms: self.timeout_ms.unwrap_or(10000),
            adaptive_max_batch_size: None,
            adaptive_min_batch_size: None,
            adaptive_max_wait_ms: None,
            adaptive_min_wait_ms: None,
            adaptive_window_secs: None,
            adaptive_enabled: None,
            webhooks: self
                .webhooks
                .as_ref()
                .map(|w| w.to_rust())
                .transpose()?,
        })
    }
}

// ---------------------------------------------------------------------------
// Source builder & source
// ---------------------------------------------------------------------------

/// Builder for configuring and constructing an ``HttpSource``.
///
/// Use ``HttpSource.builder(id)`` or ``HttpSourceBuilder(id)`` to create.
#[pyclass(name = "HttpSourceBuilder")]
pub struct PyHttpSourceBuilder {
    inner: Option<HttpSourceBuilder>,
}

#[pymethods]
impl PyHttpSourceBuilder {
    /// Create a new HttpSourceBuilder.
    ///
    /// Args:
    ///     id: Unique identifier for this source.
    #[new]
    fn new(id: &str) -> Self {
        Self {
            inner: Some(HttpSourceBuilder::new(id)),
        }
    }

    /// Set the HTTP server bind address.
    fn with_host(&mut self, host: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_host(host));
        Ok(())
    }

    /// Set the HTTP server port.
    fn with_port(&mut self, port: u16) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_port(port));
        Ok(())
    }

    /// Set the base endpoint path.
    fn with_endpoint(&mut self, endpoint: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_endpoint(endpoint));
        Ok(())
    }

    /// Set the request timeout in milliseconds.
    fn with_timeout_ms(&mut self, timeout_ms: u64) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_timeout_ms(timeout_ms));
        Ok(())
    }

    /// Set whether the source starts automatically when added to a ``DrasiLib``.
    fn with_auto_start(&mut self, auto_start: bool) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        self.inner = Some(builder.with_auto_start(auto_start));
        Ok(())
    }

    /// Configure the source from a JSON string matching the HttpSourceConfig schema.
    /// Supports the full configuration including webhook mode.
    fn with_config_json(&mut self, json_str: &str) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        let config: HttpSourceConfig =
            serde_json::from_str(json_str).map_err(|e| {
                pyo3::exceptions::PyValueError::new_err(format!("Invalid config JSON: {e}"))
            })?;
        self.inner = Some(builder.with_config(config));
        Ok(())
    }

    /// Configure the source from a typed ``HttpSourceConfig`` object.
    fn with_config(&mut self, config: &PyHttpSourceConfig) -> PyResult<()> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        let rust_config = config.to_rust()?;
        self.inner = Some(builder.with_config(rust_config));
        Ok(())
    }

    /// Consume the builder and return a configured ``HttpSource``.
    fn build(&mut self) -> PyResult<PyHttpSource> {
        let builder = self.inner.take().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Builder already consumed")
        })?;
        let source = builder.build().map_err(map_err)?;
        Ok(PyHttpSource {
            inner: Mutex::new(Some(source)),
        })
    }
}

/// A source that receives graph changes via HTTP webhooks.
#[pyclass(name = "HttpSource")]
pub struct PyHttpSource {
    inner: Mutex<Option<HttpSource>>,
}

#[pymethods]
impl PyHttpSource {
    /// Create a new ``HttpSourceBuilder`` for the given source id.
    #[staticmethod]
    fn builder(id: &str) -> PyHttpSourceBuilder {
        PyHttpSourceBuilder::new(id)
    }

    /// Consume the source and return a PyCapsule for use with ``DrasiLibBuilder``.
    fn into_source_wrapper(&self, py: Python<'_>) -> PyResult<PyObject> {
        let source = self
            .inner
            .lock()
            .map_err(map_err)?
            .take()
            .ok_or_else(|| {
                pyo3::exceptions::PyRuntimeError::new_err("Source already consumed")
            })?;
        source_to_capsule(py, source)
    }
}

/// Python module for drasi_source_http
#[pymodule]
fn _drasi_source_http(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyHttpSourceBuilder>()?;
    m.add_class::<PyHttpSource>()?;
    m.add_class::<PyHttpSourceConfig>()?;
    m.add_class::<PyWebhookConfig>()?;
    m.add_class::<PyCorsConfig>()?;
    m.add_class::<PyWebhookRoute>()?;
    m.add_class::<PyWebhookMapping>()?;
    m.add_class::<PyMappingCondition>()?;
    m.add_class::<PyElementTemplate>()?;
    m.add_class::<PyAuthConfig>()?;
    m.add_class::<PySignatureConfig>()?;
    m.add_class::<PyBearerConfig>()?;
    Ok(())
}
