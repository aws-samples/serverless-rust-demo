## Serverless Rust Demo

![build](https://github.com/aws-samples/serverless-rust-demo/actions/workflows/ci.yml/badge.svg)

<p align="center">
  <img src="imgs/diagram.png" alt="Architecture diagram"/>
</p>

This is a simple serverless application built in Rust. It consists of an API Gateway backed by four Lambda functions and a DynamoDB table for storage.

This single crate will create [four different binaries](./src/bin), one for each Lambda function. It uses an [hexagonal architecture pattern](https://aws.amazon.com/blogs/compute/developing-evolutionary-architecture-with-aws-lambda/) to decouple the [entry points](./src/bin), from the main [domain logic](./src/lib.rs), and the [storage layer](./src/store).

### Requirements

* [Rust](https://www.rust-lang.org/)
* [Cross](https://github.com/rust-embedded/cross)
* The [AWS SAM CLI](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/serverless-sam-cli-install.html)

### Usage

```bash
# Compile and prepare Lambda functions
make build

# Deploy the functions on AWS
make deploy

# Run local and integration tests against the API in the cloud
make tests
```

## Security

See [CONTRIBUTING](CONTRIBUTING.md#security-issue-notifications) for more information.

## License

This library is licensed under the MIT-0 License. See the LICENSE file.

