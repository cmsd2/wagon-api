import { Construct } from 'constructs';
import * as cdk from "aws-cdk-lib";
import * as lambda from "aws-cdk-lib/aws-lambda";
import * as iam from "aws-cdk-lib/aws-iam";
import * as path from "path";
import * as apigw from "aws-cdk-lib/aws-apigateway";
import * as cw from "aws-cdk-lib/aws-cloudwatch";
import * as ssm from "aws-cdk-lib/aws-ssm";
import * as logs from "aws-cdk-lib/aws-logs";
import * as ddb from "aws-cdk-lib/aws-dynamodb";
import { DashboardStack } from "./dashboard-stack";
import { WagonApiStack } from "./wagon-api-stack";

export interface LogsWidgetStackProps extends cdk.StackProps {
  dashboard: cw.Dashboard,
  logGroupNames: Array<string>,
}

export class LogsWidgetStack extends cdk.Stack {
    tokensTable: ddb.Table;
    handler: lambda.Function;

    constructor(scope: Construct, id: string, props: LogsWidgetStackProps) {
        super(scope, id, props);

        props.dashboard.addWidgets(new cw.LogQueryWidget({
            logGroupNames: props.logGroupNames,
            title: "Wagon Api Logs",
            width: 24,
            queryLines: [
              "fields @timestamp, @message",
              "sort @timestamp desc",
              "limit 200",
            ]
          }));
        
    }
}
