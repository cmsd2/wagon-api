import * as cdk from "@aws-cdk/core";
import * as lambda from "@aws-cdk/aws-lambda";
import * as iam from "@aws-cdk/aws-iam";
import * as path from "path";
import * as apigw from "@aws-cdk/aws-apigateway";
import * as dynamodb from "@aws-cdk/aws-dynamodb";
import { CfnOutput } from "@aws-cdk/core";

export class IndexerStack extends cdk.Stack {
    registries_table: dynamodb.ITable;
    packages_table: dynamodb.ITable;

  constructor(scope: cdk.Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    this.registries_table = new dynamodb.Table(this, id + "RegistriesTable", {
        billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
        partitionKey: { name: 'url', type: dynamodb.AttributeType.STRING },
    });

    this.packages_table = new dynamodb.Table(this, id + "PackagesTable", {
        billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
        partitionKey: { name: 'name', type: dynamodb.AttributeType.STRING },
        sortKey: { name: 'version', type: dynamodb.AttributeType.STRING },
    });
  }
}
