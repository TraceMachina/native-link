// Copyright 2022 The Native Link Authors. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// / Request object for keep alive requests.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KeepAliveRequest {
    /// / ID of the worker making the request.
    #[prost(string, tag = "1")]
    pub worker_id: ::prost::alloc::string::String,
}
/// / Request object for going away requests.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GoingAwayRequest {
    /// / ID of the worker making the request.
    #[prost(string, tag = "1")]
    pub worker_id: ::prost::alloc::string::String,
}
/// / Represents the initial request sent to the scheduler informing the
/// / scheduler about this worker's capabilities.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SupportedProperties {
    /// / The list of properties this worker can support. The exact
    /// / implementation is driven by the configuration matrix between the
    /// / worker and scheduler.
    /// /
    /// / The scheduler may reject this worker if any property keys that
    /// / the scheduler is not configured to support, or may simply ignore
    /// / the unsupported properties.
    /// /
    /// / The details on how to use this property can be found here:
    /// / <https://github.com/tracemachina/native-link/blob/c91f61edf182f2b64451fd48a5e63fa506a43aae/config/cas_server.rs>
    #[prost(message, repeated, tag = "1")]
    pub properties: ::prost::alloc::vec::Vec<
        super::super::super::super::super::build::bazel::remote::execution::v2::platform::Property,
    >,
}
/// / The result of an ExecutionRequest.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExecuteResult {
    /// / ID of the worker making the request.
    #[prost(string, tag = "1")]
    pub worker_id: ::prost::alloc::string::String,
    /// / The `instance_name` this task was initially assigned to. This is set by the client
    /// / that initially sent the job as part of the BRE protocol.
    #[prost(string, tag = "6")]
    pub instance_name: ::prost::alloc::string::String,
    /// / The original execution digest request for this response. The scheduler knows what it
    /// / should be, but we do safety checks to ensure it really is the request we expected.
    #[prost(message, optional, tag = "2")]
    pub action_digest: ::core::option::Option<
        super::super::super::super::super::build::bazel::remote::execution::v2::Digest,
    >,
    /// / The salt originally sent along with the StartExecute request. This salt is used
    /// / as a seed for cases where the execution digest should never be cached or merged
    /// / with other jobs. This salt is added to the hash function used to compute jobs that
    /// / are running or cached.
    #[prost(uint64, tag = "3")]
    pub salt: u64,
    /// / The actual response data.
    #[prost(oneof = "execute_result::Result", tags = "4, 5")]
    pub result: ::core::option::Option<execute_result::Result>,
}
/// Nested message and enum types in `ExecuteResult`.
pub mod execute_result {
    /// / The actual response data.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Result {
        /// / Result of the execution. See `build.bazel.remote.execution.v2.ExecuteResponse`
        /// / for details.
        #[prost(message, tag = "4")]
        ExecuteResponse(
            super::super::super::super::super::super::build::bazel::remote::execution::v2::ExecuteResponse,
        ),
        /// / An internal error. This is only present when an internal error happened that
        /// / was not recoverable. If the execution job failed but at no fault of the worker
        /// / it should not use this field and should send the error via execute_response.
        #[prost(message, tag = "5")]
        InternalError(super::super::super::super::super::super::google::rpc::Status),
    }
}
/// / Result sent back from the server when a node connects.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConnectionResult {
    /// / The internal ID given to the newly connected node.
    #[prost(string, tag = "1")]
    pub worker_id: ::prost::alloc::string::String,
}
/// / Communication from the scheduler to the worker.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateForWorker {
    #[prost(oneof = "update_for_worker::Update", tags = "1, 2, 3, 4, 5")]
    pub update: ::core::option::Option<update_for_worker::Update>,
}
/// Nested message and enum types in `UpdateForWorker`.
pub mod update_for_worker {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Update {
        /// / This will be sent only as the first item in the stream after the node
        /// / has connected.
        #[prost(message, tag = "1")]
        ConnectionResult(super::ConnectionResult),
        /// / Message used to let the worker know that it is still alive as well
        /// / as check to see if the worker is still alive. The worker
        /// / may close the connection if the scheduler has not sent any messages
        /// / after some amount of time (configured in the scheduler's
        /// / configuration).
        #[prost(message, tag = "2")]
        KeepAlive(()),
        /// / Informs the worker about some work it should begin performing the
        /// / requested action.
        #[prost(message, tag = "3")]
        StartAction(super::StartExecute),
        /// / Informs the worker that it has been disconnected from the pool.
        /// / The worker may discard any outstanding work that is being executed.
        #[prost(message, tag = "4")]
        Disconnect(()),
        /// / Informs the worker that it needs has been drained from the pool.
        /// / The worker no longer accepts incoming new jobs.
        #[prost(message, tag = "5")]
        Drain(()),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StartExecute {
    /// / The action information used to execute job.
    #[prost(message, optional, tag = "1")]
    pub execute_request: ::core::option::Option<
        super::super::super::super::super::build::bazel::remote::execution::v2::ExecuteRequest,
    >,
    /// / See documentation in ExecuteResult::salt.
    #[prost(uint64, tag = "2")]
    pub salt: u64,
    /// / The time at which the command was added to the queue to allow population
    /// / of the ActionResult.
    #[prost(message, optional, tag = "3")]
    pub queued_timestamp: ::core::option::Option<::prost_types::Timestamp>,
}
/// / This is a special message used to save actions into the CAS that can be used
/// / by programs like bb_browswer to inspect the history of a build.
/// / Note: Ensure this is always compatible with:
/// / <https://github.com/buildbarn/bb-remote-execution/blob/e95e066eb624dc9099682394ec18c12e218e8fc4/pkg/proto/cas/cas.proto#L23>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HistoricalExecuteResponse {
    #[prost(message, optional, tag = "1")]
    pub action_digest: ::core::option::Option<
        super::super::super::super::super::build::bazel::remote::execution::v2::Digest,
    >,
    #[prost(message, optional, tag = "3")]
    pub execute_response: ::core::option::Option<
        super::super::super::super::super::build::bazel::remote::execution::v2::ExecuteResponse,
    >,
}
/// Generated client implementations.
pub mod worker_api_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// / This API describes how schedulers communicate with Worker nodes.
    /// /
    /// / When a worker node comes online it must be pre-configured with the
    /// / endpoint of the scheduler it will register with. Once the worker
    /// / connects to the scheduler it must send a `RegisterSupportedProperties`
    /// / command to the scheduler. The scheduler will then use this information
    /// / to determine which jobs the worker can process.
    #[derive(Debug, Clone)]
    pub struct WorkerApiClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl WorkerApiClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> WorkerApiClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> WorkerApiClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            WorkerApiClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /// / Registers this worker and informs the scheduler what properties
        /// / this worker supports. The response must be listened on the client
        /// / side for updates from the server. The first item sent will always be
        /// / a ConnectionResult, after that it is undefined.
        pub async fn connect_worker(
            &mut self,
            request: impl tonic::IntoRequest<super::SupportedProperties>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::UpdateForWorker>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/com.github.trace_machina.native_link.remote_execution.WorkerApi/ConnectWorker",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "com.github.trace_machina.native_link.remote_execution.WorkerApi",
                        "ConnectWorker",
                    ),
                );
            self.inner.server_streaming(req, path, codec).await
        }
        /// / Message used to let the scheduler know that it is still alive as
        /// / well as check to see if the scheduler is still alive. The scheduler
        /// / may close the connection if the worker has not sent any messages
        /// / after some amount of time (configured in the scheduler's
        /// / configuration).
        pub async fn keep_alive(
            &mut self,
            request: impl tonic::IntoRequest<super::KeepAliveRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/com.github.trace_machina.native_link.remote_execution.WorkerApi/KeepAlive",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "com.github.trace_machina.native_link.remote_execution.WorkerApi",
                        "KeepAlive",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// / Informs the scheduler that the service is going offline and
        /// / should stop issuing any new actions on this worker.
        /// /
        /// / The worker may stay connected even after sending this command
        /// / and may even send an `ExecuteResult` after sending this command.
        /// / It is up to the scheduler implementation to decide how to handle
        /// / this case.
        /// /
        /// / Any job that was running on this instance likely needs to be
        /// / executed again, but up to the scheduler on how or when to handle
        /// / this case.
        pub async fn going_away(
            &mut self,
            request: impl tonic::IntoRequest<super::GoingAwayRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/com.github.trace_machina.native_link.remote_execution.WorkerApi/GoingAway",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "com.github.trace_machina.native_link.remote_execution.WorkerApi",
                        "GoingAway",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// / Informs the scheduler about the result of an execution request.
        pub async fn execution_response(
            &mut self,
            request: impl tonic::IntoRequest<super::ExecuteResult>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/com.github.trace_machina.native_link.remote_execution.WorkerApi/ExecutionResponse",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "com.github.trace_machina.native_link.remote_execution.WorkerApi",
                        "ExecutionResponse",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod worker_api_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with WorkerApiServer.
    #[async_trait]
    pub trait WorkerApi: Send + Sync + 'static {
        /// Server streaming response type for the ConnectWorker method.
        type ConnectWorkerStream: futures_core::Stream<
                Item = std::result::Result<super::UpdateForWorker, tonic::Status>,
            >
            + Send
            + 'static;
        /// / Registers this worker and informs the scheduler what properties
        /// / this worker supports. The response must be listened on the client
        /// / side for updates from the server. The first item sent will always be
        /// / a ConnectionResult, after that it is undefined.
        async fn connect_worker(
            &self,
            request: tonic::Request<super::SupportedProperties>,
        ) -> std::result::Result<
            tonic::Response<Self::ConnectWorkerStream>,
            tonic::Status,
        >;
        /// / Message used to let the scheduler know that it is still alive as
        /// / well as check to see if the scheduler is still alive. The scheduler
        /// / may close the connection if the worker has not sent any messages
        /// / after some amount of time (configured in the scheduler's
        /// / configuration).
        async fn keep_alive(
            &self,
            request: tonic::Request<super::KeepAliveRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status>;
        /// / Informs the scheduler that the service is going offline and
        /// / should stop issuing any new actions on this worker.
        /// /
        /// / The worker may stay connected even after sending this command
        /// / and may even send an `ExecuteResult` after sending this command.
        /// / It is up to the scheduler implementation to decide how to handle
        /// / this case.
        /// /
        /// / Any job that was running on this instance likely needs to be
        /// / executed again, but up to the scheduler on how or when to handle
        /// / this case.
        async fn going_away(
            &self,
            request: tonic::Request<super::GoingAwayRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status>;
        /// / Informs the scheduler about the result of an execution request.
        async fn execution_response(
            &self,
            request: tonic::Request<super::ExecuteResult>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status>;
    }
    /// / This API describes how schedulers communicate with Worker nodes.
    /// /
    /// / When a worker node comes online it must be pre-configured with the
    /// / endpoint of the scheduler it will register with. Once the worker
    /// / connects to the scheduler it must send a `RegisterSupportedProperties`
    /// / command to the scheduler. The scheduler will then use this information
    /// / to determine which jobs the worker can process.
    #[derive(Debug)]
    pub struct WorkerApiServer<T: WorkerApi> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: WorkerApi> WorkerApiServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for WorkerApiServer<T>
    where
        T: WorkerApi,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/com.github.trace_machina.native_link.remote_execution.WorkerApi/ConnectWorker" => {
                    #[allow(non_camel_case_types)]
                    struct ConnectWorkerSvc<T: WorkerApi>(pub Arc<T>);
                    impl<
                        T: WorkerApi,
                    > tonic::server::ServerStreamingService<super::SupportedProperties>
                    for ConnectWorkerSvc<T> {
                        type Response = super::UpdateForWorker;
                        type ResponseStream = T::ConnectWorkerStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SupportedProperties>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).connect_worker(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ConnectWorkerSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/com.github.trace_machina.native_link.remote_execution.WorkerApi/KeepAlive" => {
                    #[allow(non_camel_case_types)]
                    struct KeepAliveSvc<T: WorkerApi>(pub Arc<T>);
                    impl<
                        T: WorkerApi,
                    > tonic::server::UnaryService<super::KeepAliveRequest>
                    for KeepAliveSvc<T> {
                        type Response = ();
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::KeepAliveRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).keep_alive(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = KeepAliveSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/com.github.trace_machina.native_link.remote_execution.WorkerApi/GoingAway" => {
                    #[allow(non_camel_case_types)]
                    struct GoingAwaySvc<T: WorkerApi>(pub Arc<T>);
                    impl<
                        T: WorkerApi,
                    > tonic::server::UnaryService<super::GoingAwayRequest>
                    for GoingAwaySvc<T> {
                        type Response = ();
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GoingAwayRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).going_away(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GoingAwaySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/com.github.trace_machina.native_link.remote_execution.WorkerApi/ExecutionResponse" => {
                    #[allow(non_camel_case_types)]
                    struct ExecutionResponseSvc<T: WorkerApi>(pub Arc<T>);
                    impl<T: WorkerApi> tonic::server::UnaryService<super::ExecuteResult>
                    for ExecutionResponseSvc<T> {
                        type Response = ();
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ExecuteResult>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).execution_response(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ExecutionResponseSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: WorkerApi> Clone for WorkerApiServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: WorkerApi> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: WorkerApi> tonic::server::NamedService for WorkerApiServer<T> {
        const NAME: &'static str = "com.github.trace_machina.native_link.remote_execution.WorkerApi";
    }
}
