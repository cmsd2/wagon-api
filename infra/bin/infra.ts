#!/usr/bin/env node
import "source-map-support/register";
import * as cdk from "@aws-cdk/core";
import { WagonApiStack } from "../lib/wagon-api-stack";
import { DashboardStack } from "../lib/dashboard-stack";
import { IndexerStack } from "../lib/indexer-stack";
import { env } from "process";
import { TokensApiStack } from "../lib/tokens-api-stack";
import { V1ApiStack } from "../lib/v1-api-stack";
import { LogsWidgetStack } from "../lib/logs-widget-stack";

const app = new cdk.App();
const dashboard = new DashboardStack(app, "WagonDashboard", {name: "wagon"});
const apiStack = new WagonApiStack(app, "WagonApi", {
    dashboard: dashboard.dashboard,
    openid_aud: process.env.OPENID_AUD!,
    openid_config_uri: process.env.OPENID_CONFIG_URI!,
});
const tokensApi = new TokensApiStack(app, "WagonTokensApi", {
    dashboard: dashboard.dashboard, 
    api: apiStack
});
const v1Api = new V1ApiStack(app, "WagonV1Api", {
    dashboard: dashboard.dashboard,
    api: apiStack
});
new IndexerStack(app, "WagonIndexer", {});
new LogsWidgetStack(app, "WagonLogsWidget", {
    dashboard: dashboard.dashboard,
    logGroupNames: [
        tokensApi.handler.logGroup.logGroupName,
        v1Api.handler.logGroup.logGroupName,
        apiStack.logGroup.logGroupName,
        apiStack.jwtAuthorizerFunction.logGroup.logGroupName,
    ]
});