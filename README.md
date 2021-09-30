## Serverless Rust Demo

This is a simple serverless application using Rust as its main programming language.

### Requirements

* [Rust](https://www.rust-lang.org/)
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

