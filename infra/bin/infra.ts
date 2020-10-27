#!/usr/bin/env node
import "source-map-support/register";
import * as cdk from "@aws-cdk/core";
import { WagonApiStack } from "../lib/wagon-api-stack";
import { DashboardStack } from "../lib/dashboard-stack";
import { IndexerStack } from "../lib/indexer-stack";
import { env } from "process";

const app = new cdk.App();
const dashboard = new DashboardStack(app, "WagonDashboard", {name: "wagon"});
new WagonApiStack(app, "WagonApi", { dashboard: dashboard.dashboard });
new IndexerStack(app, "WagonIndexer", {});