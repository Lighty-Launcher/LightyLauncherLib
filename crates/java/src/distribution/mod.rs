// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Java distribution URL routing (Temurin, GraalVM, Zulu, Liberica).

mod api_models;
mod providers;
mod dispatcher;

pub(crate) use dispatcher::get_download_url;
