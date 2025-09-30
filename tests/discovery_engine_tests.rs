use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn get_discovery_script() -> PathBuf {
    PathBuf::from("./vortex_discovery")
}

fn run_discovery(args: &[&str]) -> Result<std::process::Output> {
    let output = Command::new(get_discovery_script()).args(args).output()?;
    Ok(output)
}

fn run_discovery_expect_success(args: &[&str]) -> Result<String> {
    let output = run_discovery(args)?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Discovery command failed: ./vortex_discovery {}\nStdout: {}\nStderr: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[test]
fn test_discovery_help() -> Result<()> {
    let output = run_discovery_expect_success(&["help"])?;

    assert!(output.contains("Vortex Discovery Help"));
    assert!(output.contains("scan"));
    assert!(output.contains("validate"));
    assert!(output.contains("Supported Languages"));
    assert!(output.contains("node"));
    assert!(output.contains("python"));
    assert!(output.contains("go"));
    assert!(output.contains("rust"));

    Ok(())
}

#[test]
fn test_node_project_detection() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a Node.js project structure
    let frontend_dir = temp_dir.path().join("frontend");
    fs::create_dir_all(&frontend_dir)?;

    fs::write(
        frontend_dir.join("package.json"),
        r#"{
            "name": "my-frontend",
            "version": "1.0.0",
            "scripts": {
                "start": "react-scripts start",
                "build": "react-scripts build"
            },
            "dependencies": {
                "react": "^18.0.0"
            }
        }"#,
    )?;

    fs::write(
        frontend_dir
            .join("src")
            .tap(|p| fs::create_dir_all(p).unwrap())
            .join("index.js"),
        "import React from 'react';",
    )?;

    let output = run_discovery_expect_success(&["scan", temp_dir.path().to_str().unwrap()])?;

    assert!(output.contains("Discovered") && output.contains("potential services"));
    assert!(output.contains("frontend"));
    assert!(output.contains("node"));
    assert!(output.contains("frontend")); // service type
    assert!(output.contains("node:18-alpine"));
    assert!(output.contains("3000"));
    assert!(output.contains("Configuration saved"));
    assert!(output.contains("vortex.yaml"));

    // Verify the configuration file was created
    let config_file = temp_dir.path().join("vortex.yaml");
    assert!(config_file.exists());

    let config_content = fs::read_to_string(config_file)?;
    assert!(config_content.contains("frontend:"));
    assert!(config_content.contains("type: frontend"));
    assert!(config_content.contains("language: node"));
    assert!(config_content.contains("image: node:18-alpine"));
    assert!(config_content.contains("3000:3000"));

    Ok(())
}

#[test]
fn test_python_project_detection() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a Python backend project
    let backend_dir = temp_dir.path().join("api");
    fs::create_dir_all(&backend_dir)?;

    fs::write(
        backend_dir.join("requirements.txt"),
        "flask==2.3.3\nfastapi==0.104.0\nuvicorn==0.24.0",
    )?;

    fs::write(
        backend_dir.join("main.py"),
        r#"
from fastapi import FastAPI
app = FastAPI()

@app.get("/")
def read_root():
    return {"Hello": "World"}
"#,
    )?;

    let output = run_discovery_expect_success(&["scan", temp_dir.path().to_str().unwrap()])?;

    assert!(output.contains("api"));
    assert!(output.contains("python"));
    assert!(output.contains("backend")); // service type inferred from 'api' directory
    assert!(output.contains("python:3.11-slim"));
    assert!(output.contains("8000"));

    // Verify configuration
    let config_file = temp_dir.path().join("vortex.yaml");
    let config_content = fs::read_to_string(config_file)?;
    assert!(config_content.contains("api:"));
    assert!(config_content.contains("type: backend"));
    assert!(config_content.contains("language: python"));
    assert!(config_content.contains("image: python:3.11-slim"));

    Ok(())
}

#[test]
fn test_go_project_detection() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a Go worker service
    let worker_dir = temp_dir.path().join("worker");
    fs::create_dir_all(&worker_dir)?;

    fs::write(
        worker_dir.join("go.mod"),
        r#"module example.com/worker

go 1.21

require (
    github.com/gorilla/mux v1.8.0
    github.com/redis/go-redis/v9 v9.0.0
)
"#,
    )?;

    fs::write(
        worker_dir.join("main.go"),
        r#"
package main

import (
    "fmt"
    "log"
    "net/http"
)

func main() {
    fmt.Println("Worker service starting...")
    log.Fatal(http.ListenAndServe(":8080", nil))
}
"#,
    )?;

    let output = run_discovery_expect_success(&["scan", temp_dir.path().to_str().unwrap()])?;

    assert!(output.contains("worker"));
    assert!(output.contains("go"));
    assert!(output.contains("worker")); // service type
    assert!(output.contains("golang:1.21-alpine"));

    let config_file = temp_dir.path().join("vortex.yaml");
    let config_content = fs::read_to_string(config_file)?;
    assert!(config_content.contains("worker:"));
    assert!(config_content.contains("type: worker"));
    assert!(config_content.contains("language: go"));

    Ok(())
}

#[test]
fn test_rust_project_detection() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a Rust service
    let service_dir = temp_dir.path().join("rust-service");
    fs::create_dir_all(&service_dir)?;

    fs::write(
        service_dir.join("Cargo.toml"),
        r#"[package]
name = "rust-service"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
axum = "0.7"
"#,
    )?;

    let src_dir = service_dir.join("src");
    fs::create_dir_all(&src_dir)?;
    fs::write(
        src_dir.join("main.rs"),
        r#"
use axum::{response::Html, routing::get, Router};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(handler));
    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, Rust!</h1>")
}
"#,
    )?;

    let output = run_discovery_expect_success(&["scan", temp_dir.path().to_str().unwrap()])?;

    assert!(output.contains("rust-service"));
    assert!(output.contains("rust"));
    assert!(output.contains("service")); // default service type
    assert!(output.contains("rust:1.70"));

    Ok(())
}

#[test]
fn test_multi_service_project_detection() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a complex multi-service project
    let services = [
        (
            "frontend",
            "package.json",
            r#"{"name": "frontend", "dependencies": {"react": "^18.0.0"}}"#,
        ),
        (
            "backend",
            "requirements.txt",
            "fastapi==0.104.0\npsycopg2==2.9.7",
        ),
        ("worker", "go.mod", "module example.com/worker\n\ngo 1.21"),
        (
            "database",
            "init.sql",
            "CREATE TABLE users (id SERIAL PRIMARY KEY);",
        ),
    ];

    for (service_name, file_name, content) in &services {
        let service_dir = temp_dir.path().join(service_name);
        fs::create_dir_all(&service_dir)?;
        fs::write(service_dir.join(file_name), content)?;

        // Add some structure to help with service type detection
        if *service_name == "frontend" {
            fs::create_dir_all(service_dir.join("src"))?;
            fs::create_dir_all(service_dir.join("public"))?;
        } else if *service_name == "backend" {
            fs::create_dir_all(service_dir.join("api"))?;
        } else if *service_name == "database" {
            fs::create_dir_all(service_dir.join("migrations"))?;
        }
    }

    let output = run_discovery_expect_success(&["scan", temp_dir.path().to_str().unwrap()])?;

    // Should detect all 4 services
    assert!(output.contains("Discovered 4 potential services"));
    assert!(output.contains("frontend"));
    assert!(output.contains("backend"));
    assert!(output.contains("worker"));
    assert!(output.contains("database"));

    // Verify service types
    assert!(output.contains("⚛️")); // frontend emoji
    assert!(output.contains("🐍")); // backend emoji
    assert!(output.contains("⚙️")); // worker emoji
    assert!(output.contains("🐘")); // database emoji

    // Verify configuration file
    let config_file = temp_dir.path().join("vortex.yaml");
    let config_content = fs::read_to_string(config_file)?;

    assert!(config_content.contains("frontend:"));
    assert!(config_content.contains("type: frontend"));
    assert!(config_content.contains("language: node"));

    assert!(config_content.contains("backend:"));
    assert!(config_content.contains("type: backend"));
    assert!(config_content.contains("language: python"));

    assert!(config_content.contains("worker:"));
    assert!(config_content.contains("type: worker"));
    assert!(config_content.contains("language: go"));

    assert!(config_content.contains("database:"));
    assert!(config_content.contains("type: database"));

    // Should have context configurations
    assert!(config_content.contains("contexts:"));
    assert!(config_content.contains("dev:"));
    assert!(config_content.contains("staging:"));
    assert!(config_content.contains("prod:"));

    Ok(())
}

#[test]
fn test_empty_directory_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create some empty directories that should be skipped
    fs::create_dir_all(temp_dir.path().join("empty1"))?;
    fs::create_dir_all(temp_dir.path().join("empty2"))?;
    fs::create_dir_all(temp_dir.path().join(".hidden"))?;
    fs::create_dir_all(temp_dir.path().join("node_modules"))?;

    let output = run_discovery(&["scan", temp_dir.path().to_str().unwrap()])?;

    // Should fail with no services detected
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No services detected")
            || String::from_utf8_lossy(&output.stdout).contains("No services detected")
    );

    Ok(())
}

#[test]
fn test_config_validation() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a valid config first
    let valid_config = r#"
name: test-project
description: Test workspace

services:
  frontend:
    type: frontend
    language: node
    image: node:18-alpine
    ports:
      - 3000:3000

contexts:
  dev:
    description: Development environment
"#;

    let config_file = temp_dir.path().join("vortex.yaml");
    fs::write(&config_file, valid_config)?;

    let output = run_discovery_expect_success(&["validate", config_file.to_str().unwrap()])?;

    assert!(output.contains("Configuration Validation"));
    assert!(output.contains("✅ Configuration file exists"));

    Ok(())
}

#[test]
fn test_php_project_detection() -> Result<()> {
    let temp_dir = TempDir::new()?;

    let service_dir = temp_dir.path().join("web");
    fs::create_dir_all(&service_dir)?;

    fs::write(
        service_dir.join("composer.json"),
        r#"{
            "name": "example/web-app",
            "require": {
                "php": "^8.2",
                "laravel/framework": "^10.0"
            }
        }"#,
    )?;

    fs::write(service_dir.join("index.php"), "<?php\necho 'Hello PHP!';\n")?;

    let output = run_discovery_expect_success(&["scan", temp_dir.path().to_str().unwrap()])?;

    assert!(output.contains("web"));
    assert!(output.contains("php"));
    assert!(output.contains("php:8.2-fpm-alpine"));

    Ok(())
}

#[test]
fn test_ruby_project_detection() -> Result<()> {
    let temp_dir = TempDir::new()?;

    let service_dir = temp_dir.path().join("rails-app");
    fs::create_dir_all(&service_dir)?;

    fs::write(
        service_dir.join("Gemfile"),
        r#"
source 'https://rubygems.org'
ruby '3.2.0'

gem 'rails', '~> 7.0'
gem 'pg', '~> 1.1'
"#,
    )?;

    fs::write(
        service_dir.join("config.ru"),
        "require_relative 'config/application'\nrun Rails.application",
    )?;

    let output = run_discovery_expect_success(&["scan", temp_dir.path().to_str().unwrap()])?;

    assert!(output.contains("rails-app"));
    assert!(output.contains("ruby"));
    assert!(output.contains("ruby:3.2-alpine"));

    Ok(())
}

#[test]
fn test_java_project_detection() -> Result<()> {
    let temp_dir = TempDir::new()?;

    let service_dir = temp_dir.path().join("spring-service");
    fs::create_dir_all(&service_dir)?;

    fs::write(
        service_dir.join("pom.xml"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>spring-service</artifactId>
    <version>1.0.0</version>
    <parent>
        <groupId>org.springframework.boot</groupId>
        <artifactId>spring-boot-starter-parent</artifactId>
        <version>3.1.0</version>
    </parent>
    <dependencies>
        <dependency>
            <groupId>org.springframework.boot</groupId>
            <artifactId>spring-boot-starter-web</artifactId>
        </dependency>
    </dependencies>
</project>"#,
    )?;

    let output = run_discovery_expect_success(&["scan", temp_dir.path().to_str().unwrap()])?;

    assert!(output.contains("spring-service"));
    assert!(output.contains("java"));
    assert!(output.contains("openjdk:17-alpine"));

    Ok(())
}

#[test]
fn test_docker_project_detection() -> Result<()> {
    let temp_dir = TempDir::new()?;

    let service_dir = temp_dir.path().join("custom-service");
    fs::create_dir_all(&service_dir)?;

    fs::write(
        service_dir.join("Dockerfile"),
        r#"FROM ubuntu:22.04
RUN apt-get update && apt-get install -y python3
COPY . /app
WORKDIR /app
CMD ["python3", "app.py"]"#,
    )?;

    fs::write(service_dir.join("app.py"), "print('Custom Docker service')")?;

    let output = run_discovery_expect_success(&["scan", temp_dir.path().to_str().unwrap()])?;

    assert!(output.contains("custom-service"));
    assert!(output.contains("docker"));
    assert!(output.contains("custom"));

    Ok(())
}

#[test]
fn test_service_type_inference_from_directory_structure() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Test various directory structures that should influence service type detection
    let test_cases = [
        ("ui", "package.json", "frontend"),
        ("client", "package.json", "frontend"),
        ("web", "package.json", "frontend"),
        ("api", "requirements.txt", "backend"),
        ("server", "go.mod", "backend"),
        ("backend", "Cargo.toml", "backend"),
        ("jobs", "package.json", "worker"),
        ("worker", "requirements.txt", "worker"),
        ("tasks", "go.mod", "worker"),
        ("cache", "requirements.txt", "cache"),
        ("redis", "go.mod", "cache"),
        ("queue", "package.json", "queue"),
        ("broker", "requirements.txt", "queue"),
        ("data", "init.sql", "database"),
        ("db", "schema.sql", "database"),
    ];

    for (dir_name, file_name, _expected_type) in &test_cases {
        let service_dir = temp_dir.path().join(dir_name);
        fs::create_dir_all(&service_dir)?;

        let content = match *file_name {
            "package.json" => r#"{"name": "test"}"#,
            "requirements.txt" => "flask==2.3.3",
            "go.mod" => "module test\n\ngo 1.21",
            "Cargo.toml" => "[package]\nname = \"test\"",
            _ => "-- test content",
        };

        fs::write(service_dir.join(file_name), content)?;
    }

    let output = run_discovery_expect_success(&["scan", temp_dir.path().to_str().unwrap()])?;

    // Verify each service type is detected correctly
    for (dir_name, _, expected_type) in &test_cases {
        // Look for the service in the output - check both the numbered list and config preview
        assert!(
            output.contains(dir_name) && output.contains(expected_type),
            "Expected to find directory '{}' with type '{}' in output, but output was:\n{}",
            dir_name,
            expected_type,
            output
        );
    }

    Ok(())
}

// Helper trait to make chaining more readable
trait DirectoryExt {
    fn tap<F>(self, f: F) -> Self
    where
        F: FnOnce(&Self);
}

impl<T> DirectoryExt for T {
    fn tap<F>(self, f: F) -> Self
    where
        F: FnOnce(&Self),
    {
        f(&self);
        self
    }
}
