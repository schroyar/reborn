# Reborn

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

* This Rust application streamlines the ingestion and processing of real-time events from Phoenix Trade, facilitating efficient data handling and integration into distributed systems. By leveraging Docker for deployment and Kafka for event messaging, coupled with Kafka UI for monitoring, it offers a comprehensive solution for scalable and fault-tolerant event streaming.

## Key Features:

* Real-Time Streaming with Buffering: Establishes a high-performance stream from Phoenix Trade, using an intelligent buffering system to accommodate fluctuating event volumes without loss of data.
* Seamless Kafka Broadcasting: Events are reliably broadcasted to Kafka, supporting distributed logging and enabling subsequent processing or analytics across decentralized systems.
* Dockerized Deployment: The application and its Kafka infrastructure are containerized with Docker, simplifying setup and ensuring consistent environments across development, testing, and production.
* Kafka UI Integration: Incorporates Kafka UI, providing a user-friendly interface for real-time monitoring of Kafka topics, messages, and system health. This enhances operational visibility and aids in the quick diagnosis of issues.
