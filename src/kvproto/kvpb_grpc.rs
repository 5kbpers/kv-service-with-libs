// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

const METHOD_KV_GET: ::grpcio::Method<super::kvrpcpb::KvReq, super::kvrpcpb::GetResp> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/kvpb.kv/Get",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_KV_PUT: ::grpcio::Method<super::kvrpcpb::KvReq, super::kvrpcpb::PutResp> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/kvpb.kv/Put",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_KV_DELETE: ::grpcio::Method<super::kvrpcpb::KvReq, super::kvrpcpb::DeleteResp> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/kvpb.kv/Delete",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_KV_RAFT: ::grpcio::Method<super::eraftpb::Message, super::kvrpcpb::RaftDone> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/kvpb.kv/Raft",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

#[derive(Clone)]
pub struct KvClient {
    client: ::grpcio::Client,
}

impl KvClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        KvClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn get_opt(&self, req: &super::kvrpcpb::KvReq, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::kvrpcpb::GetResp> {
        self.client.unary_call(&METHOD_KV_GET, req, opt)
    }

    pub fn get(&self, req: &super::kvrpcpb::KvReq) -> ::grpcio::Result<super::kvrpcpb::GetResp> {
        self.get_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_async_opt(&self, req: &super::kvrpcpb::KvReq, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::kvrpcpb::GetResp>> {
        self.client.unary_call_async(&METHOD_KV_GET, req, opt)
    }

    pub fn get_async(&self, req: &super::kvrpcpb::KvReq) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::kvrpcpb::GetResp>> {
        self.get_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn put_opt(&self, req: &super::kvrpcpb::KvReq, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::kvrpcpb::PutResp> {
        self.client.unary_call(&METHOD_KV_PUT, req, opt)
    }

    pub fn put(&self, req: &super::kvrpcpb::KvReq) -> ::grpcio::Result<super::kvrpcpb::PutResp> {
        self.put_opt(req, ::grpcio::CallOption::default())
    }

    pub fn put_async_opt(&self, req: &super::kvrpcpb::KvReq, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::kvrpcpb::PutResp>> {
        self.client.unary_call_async(&METHOD_KV_PUT, req, opt)
    }

    pub fn put_async(&self, req: &super::kvrpcpb::KvReq) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::kvrpcpb::PutResp>> {
        self.put_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn delete_opt(&self, req: &super::kvrpcpb::KvReq, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::kvrpcpb::DeleteResp> {
        self.client.unary_call(&METHOD_KV_DELETE, req, opt)
    }

    pub fn delete(&self, req: &super::kvrpcpb::KvReq) -> ::grpcio::Result<super::kvrpcpb::DeleteResp> {
        self.delete_opt(req, ::grpcio::CallOption::default())
    }

    pub fn delete_async_opt(&self, req: &super::kvrpcpb::KvReq, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::kvrpcpb::DeleteResp>> {
        self.client.unary_call_async(&METHOD_KV_DELETE, req, opt)
    }

    pub fn delete_async(&self, req: &super::kvrpcpb::KvReq) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::kvrpcpb::DeleteResp>> {
        self.delete_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn raft_opt(&self, req: &super::eraftpb::Message, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::kvrpcpb::RaftDone> {
        self.client.unary_call(&METHOD_KV_RAFT, req, opt)
    }

    pub fn raft(&self, req: &super::eraftpb::Message) -> ::grpcio::Result<super::kvrpcpb::RaftDone> {
        self.raft_opt(req, ::grpcio::CallOption::default())
    }

    pub fn raft_async_opt(&self, req: &super::eraftpb::Message, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::kvrpcpb::RaftDone>> {
        self.client.unary_call_async(&METHOD_KV_RAFT, req, opt)
    }

    pub fn raft_async(&self, req: &super::eraftpb::Message) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::kvrpcpb::RaftDone>> {
        self.raft_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Item = (), Error = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait Kv {
    fn get(&mut self, ctx: ::grpcio::RpcContext, req: super::kvrpcpb::KvReq, sink: ::grpcio::UnarySink<super::kvrpcpb::GetResp>);
    fn put(&mut self, ctx: ::grpcio::RpcContext, req: super::kvrpcpb::KvReq, sink: ::grpcio::UnarySink<super::kvrpcpb::PutResp>);
    fn delete(&mut self, ctx: ::grpcio::RpcContext, req: super::kvrpcpb::KvReq, sink: ::grpcio::UnarySink<super::kvrpcpb::DeleteResp>);
    fn raft(&mut self, ctx: ::grpcio::RpcContext, req: super::eraftpb::Message, sink: ::grpcio::UnarySink<super::kvrpcpb::RaftDone>);
}

pub fn create_kv<S: Kv + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_KV_GET, move |ctx, req, resp| {
        instance.get(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_KV_PUT, move |ctx, req, resp| {
        instance.put(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_KV_DELETE, move |ctx, req, resp| {
        instance.delete(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_KV_RAFT, move |ctx, req, resp| {
        instance.raft(ctx, req, resp)
    });
    builder.build()
}
