# Copyright 2022 The NativeLink Authors. All rights reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#    http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# --- Begin Lambda Function to update internal scheduler ips ---

# This lambda will be configured to listen to the scheduler auto-scaling group
# so, when new instances are created/destroyed it will trigger this lambda.
# The lambda will query all instances that are part of the auto scaling group
# and add the private IPs of each instance to a Route53 DNS record that will
# be used by workers to find the scheduler(s).
resource "aws_lambda_function" "update_scheduler_ips_lambda" {
  function_name = "nativelink_update_scheduler_ips"

  filename = data.archive_file.update_scheduler_ips.output_path
  role = aws_iam_role.update_scheduler_ips_lambda_role.arn
  handler = "update_scheduler_ips.lambda_handler"
  runtime = "python3.8"
  source_code_hash = data.archive_file.update_scheduler_ips.output_base64sha256
  timeout = 30

  environment {
    variables = {
      HOSTED_ZONE_ID = data.aws_route53_zone.main.zone_id,
      SCHEDULER_DOMAIN = "${var.scheduler_domain_prefix}.internal.${var.base_domain}",
      AUTO_SCALING_GROUP_NAME = aws_autoscaling_group.scheduler_autoscaling_group.name
    }
  }
}

data "archive_file" "update_scheduler_ips" {
  type = "zip"
  source_file = "scripts/lambdas/update_scheduler_ips.py"
  output_path = ".update_scheduler_ips.zip"
}

resource "aws_sns_topic" "update_scheduler_ips_sns_topic" {
  name = "nativelink_scheduler_ips_sns_topic"
}

resource "aws_sns_topic_subscription" "update_scheduler_ips_sns_subscription" {
  endpoint = aws_lambda_function.update_scheduler_ips_lambda.arn
  protocol = "lambda"
  topic_arn = aws_sns_topic.update_scheduler_ips_sns_topic.arn
}

resource "aws_lambda_permission" "allow_scheduler_ip_sns_execute_lambda" {
  statement_id  = "nativelink_allow_scheduler_ip_sns_execute_lambda"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.update_scheduler_ips_lambda.arn
  principal     = "sns.amazonaws.com"
  source_arn = aws_sns_topic.update_scheduler_ips_sns_topic.arn
}

# --- End Lambda Function to update internal scheduler ips ---
