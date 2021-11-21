## Serverless Rust Demo

![build](https://github.com/aws-samples/serverless-rust-demo/actions/workflows/ci.yml/badge.svg)

<p align="center">
  <img src="imgs/diagram.png" alt="Architecture diagram"/>
</p>

This is a simple serverless application built in Rust. It consists of an API Gateway backed by four Lambda functions and a DynamoDB table for storage.

This single crate will create [five different binaries](./src/bin), one for each Lambda function. It uses an [hexagonal architecture pattern](https://aws.amazon.com/blogs/compute/developing-evolutionary-architecture-with-aws-lambda/) to decouple the [entry points](./src/bin), from the main [domain logic](./src/lib.rs), the [storage component](./src/store), and the [event bus component](./src/event_bus).

You can find a walkthrough of the code in this project on [the AWS Twitch channel](https://www.twitch.tv/videos/1201473601).

### Requirements

* [Rust](https://www.rust-lang.org/)
* [Cross](https://github.com/rust-embedded/cross) for cross-compilation to Arm64
* The [AWS SAM CLI](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/serverless-sam-cli-install.html) for deploying to the cloud
* [Artillery](https://artillery.io/) for load-testing the application

### Usage

```bash
# Run unit tests
make tests-unit

# Compile and prepare Lambda functions
make build

# Deploy the functions on AWS
make deploy

# Run integration tests against the API in the cloud
make tests-integ

# Run a load test against the API in the cloud
make tests-load
```

## Security

See [CONTRIBUTING](CONTRIBUTING.md#security-issue-notifications) for more information.

## License

This library is licensed under the MIT-0 License. See the LICENSE file.

