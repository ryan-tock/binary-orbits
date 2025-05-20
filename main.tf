provider "aws" {
    region = "us-west-2"
}

resource "aws_iam_role" "lambda_role" {
    name = "lambda-execution-role"
    assume_role_policy = jsonencode({
        "Version": "2012-10-17",
        "Statement": [
            {
                "Effect": "Allow",
                "Principal": {
                    "Service": "lambda.amazonaws.com"
                },
                "Action": "sts:AssumeRole"
            }
        ]
    })
}

resource "aws_lambda_function" "optimize_orbit" {
    filename = "lambda.zip"
    function_name = "optimize_orbit"
    handler = "lambda_function.lambda_handler"
    runtime = "python3.13"
    role = aws_iam_role.lambda_role.arn
    source_code_hash = filebase64sha256("${path.module}/lambda.zip")

    architectures = [
        "arm64"
    ]

    layers = [
        aws_lambda_layer_version.scipy_layer.arn
    ]

    timeout = 120
    memory_size = 1769
}

resource "aws_lambda_permission" "apigateway_invoke" {
  statement_id  = "AllowAPIGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.optimize_orbit.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.http_api.execution_arn}/*"
}

resource "aws_lambda_layer_version" "scipy_layer" {
  layer_name          = "scipy-layer"
  filename            = "scipy-layer.zip"
  compatible_runtimes = ["python3.13"]
  compatible_architectures = ["arm64"]

  source_code_hash = filebase64sha256("${path.module}/scipy-layer.zip")
}

resource "aws_apigatewayv2_api" "http_api" {
    name = "optimize-orbit-api"
    protocol_type = "HTTP"
    cors_configuration {
        allow_origins = ["http://localhost:8000", "https://ryan-tock.github.io"]
        allow_methods = ["POST", "GET", "OPTIONS"]
        allow_headers = ["content-type"]
        max_age = 300
    }
}

resource "aws_apigatewayv2_integration" "lambda_integration" {
    api_id = aws_apigatewayv2_api.http_api.id
    integration_type = "AWS_PROXY"
    integration_uri = aws_lambda_function.optimize_orbit.invoke_arn
    payload_format_version = "2.0"
}

resource "aws_apigatewayv2_route" "process_route" {
    api_id = aws_apigatewayv2_api.http_api.id
    route_key = "POST /process"
    target = "integrations/${aws_apigatewayv2_integration.lambda_integration.id}"
}

resource "aws_apigatewayv2_stage" "default_stage" {
    api_id = aws_apigatewayv2_api.http_api.id
    name = "$default"
    auto_deploy = true
}

output "api_endpoint" {
    value = aws_apigatewayv2_stage.default_stage.invoke_url
}