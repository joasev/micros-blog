# Rust Microservices Application

## Overview

This Rust Microservices Application is designed to showcase the development and deployment of a microservices architecture using Rust. It utilizes Docker to create distinct compilation and runtime images, ensuring that the runtime image remains lightweight. Additionally, it demonstrates the deployment and basic networking of these microservices using Kubernetes.

## Features

- **Microservices Architecture**: Implements a microservices architecture to ensure modularity and scalability.
- **Optimized Docker Image Building**: Utilizes Docker to build separate compilation and runtime images, optimizing for a lightweight production environment.
- **Kubernetes Deployment**: DDemonstrates the deployment of microservices on Kubernetes, including the setup of Deployments, Services, and an ingress controller for managing external access.
- **RabbitMQ**: Leverages a containerized instance of RabbitMQ to act as an event bus for inter-service communication, ensuring reliable message delivery and decoupling of services.
- **MongoDB**: Uses MongoDB as a database to store and manage processed events, facilitating efficient query handling and data retrieval.
