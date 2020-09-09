#!/usr/bin/env node
import "source-map-support/register";
import * as cdk from "@aws-cdk/core";
import { WagonApiStack } from "../lib/wagon-api-stack";

const app = new cdk.App();
new WagonApiStack(app, "WagonApi");
