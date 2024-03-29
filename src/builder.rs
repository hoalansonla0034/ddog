//! Exposed Query Builder

use crate::{routes, types};

/// Builder for creating datadog API requests
///
/// ## Usage
///
/// Below we showcase using the ddog::Builder to post metrics to the datadog API.
///
/// ```rust
/// use ddog::prelude::*;
///
/// async {
///     let mut builder = builder::Builder::new();
///     let (status, res) = builder.v2()
///         .create_new_tag_config("my.metric.name")
///         .headers(vec![
///             ("Accept", "application/json"),
///             ("Content-Type", "application/json"),
///             ("DD-API-KEY", "<api_key>"),
///             ("DD-APPLICATION-KEY", "<application_key>"),
///         ])
///         .execute().await;
///
///     // This should return a 403 status code now since the above API key is invalid.
///     println!("Status Code: {:?}", status);
///     println!("Response: {:?}", res);
/// };
/// ```
#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct Builder {
    /// API Version
    pub version: types::version::ApiVersion,
    /// Request headers
    pub headers: Vec<(String, String)>,
}

impl Builder {
    /// Initializes the query builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate the json body
    ///
    /// Return some error if not valid.
    pub fn is_body_valid_json(body: &str) -> Option<serde_json::Error> {
        match serde_json::from_str::<serde_json::value::Value>(body) {
            Ok(_) => None,
            Err(e) => Some(e),
        }
    }

    /// Initialize a tracing subscriber
    pub fn with_subscriber(&mut self) -> &mut Self {
        let subscriber_builder = tracing_subscriber::fmt();
        let mut env_filter = tracing_subscriber::EnvFilter::from_default_env();
        env_filter = env_filter.add_directive(tracing::Level::DEBUG.into());
        if let Err(e) = subscriber_builder.with_env_filter(env_filter).try_init() {
            println!("Failed to initialize tracing!\nError: {:?}", e)
        }
        self
    }

    /// Sets the api version to v1
    pub fn v1(&mut self) -> &mut Self {
        self.version = types::version::ApiVersion::V1;
        self
    }

    /// Sets the api version to v2
    pub fn v2(&mut self) -> &mut Self {
        self.version = types::version::ApiVersion::V2;
        self
    }

    /// Creates the respective route for the given route enum
    // pub fn route<T>(&mut self, route: Route) -> impl Route<T>
    // where
    //     // routes::metrics::tags::Tags: types::route::Route<T>,
    //     T: std::fmt::Debug,
    // {
    //     match self.version {
    //         ApiVersion::V2 => match version {
    //             // V2Routes::Metrics => self.metrics(),
    //             V2Routes::Metrics => panic!("Not implemented!"),
    //         },
    //         _ => panic!("Invalid Route Version \"V2Routes\" after calling builder.v1()"),
    //     }
    // }

    /// Create a new Tag Configuration
    pub fn create_new_tag_config<T>(&self, metric_name: &str) -> impl types::route::Route<T>
    where
        routes::metrics::tags::Tags: types::route::Route<T>,
        T: std::fmt::Debug,
    {
        match self.version {
            types::version::ApiVersion::V2 => routes::metrics::tags::Tags::new(metric_name),
            _ => panic!("Unimplemented API Version"),
        }
    }

    /// Posts series data to the metrics endpoint
    pub fn post_series<T>(&self) -> impl types::route::Route<T>
    where
        routes::metrics::series::Series: types::route::Route<T>,
        T: std::fmt::Debug,
    {
        match self.version {
            types::version::ApiVersion::V2 => routes::metrics::series::Series::new(),
            _ => panic!("Unimplemented API Version"),
        }
    }

    /// Posts distribution points to the metrics endpoint
    pub fn post_distribution<T>(&self) -> impl types::route::Route<T>
    where
        routes::metrics::distribution::Distribution: types::route::Route<T>,
        T: std::fmt::Debug,
    {
        match routes::metrics::distribution::Distribution::try_from(self.version) {
            Ok(distribution) => distribution,
            Err(e) => {
                tracing::error!(target: "builder", "Failed to create distribution for api version: {:?} with error: {:?}", self.version, e);
                panic!("Unimplemented API Version: {:?}", e)
            }
        }
    }

    /// Gets a list of active metrics
    pub fn get_metrics<T>(
        &self,
        from: usize,
        host: Option<String>,
        tag_filter: Option<String>,
    ) -> impl types::route::Route<T>
    where
        routes::metrics::get_metrics::GetMetrics: types::route::Route<T>,
        T: std::fmt::Debug,
    {
        match routes::metrics::get_metrics::GetMetrics::try_from(self.version) {
            Ok(metrics) => metrics
                .set_from(from)
                .set_host(host.unwrap_or_else(|| "".to_string()))
                .set_tag_filter(tag_filter.unwrap_or_else(|| "".to_string())),
            Err(e) => {
                tracing::error!(target: "builder", "Failed to create metrics for api version: {:?} with error: {:?}", self.version, e);
                panic!("Unimplemented API Version: {:?}", e)
            }
        }
    }
}
