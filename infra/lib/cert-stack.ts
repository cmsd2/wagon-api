import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';
// import * as sqs from 'aws-cdk-lib/aws-sqs';
import * as s3 from 'aws-cdk-lib/aws-s3';
import { Asset } from 'aws-cdk-lib/aws-s3-assets';
import { BucketDeployment, Source } from 'aws-cdk-lib/aws-s3-deployment';
import * as cloudfront from 'aws-cdk-lib/aws-cloudfront';
import * as origins from 'aws-cdk-lib/aws-cloudfront-origins';
import * as acm from 'aws-cdk-lib/aws-certificatemanager';
import * as route53 from 'aws-cdk-lib/aws-route53';
import * as targets from 'aws-cdk-lib/aws-route53-targets';
import * as apigw from 'aws-cdk-lib/aws-apigateway';
import * as ssm from 'aws-cdk-lib/aws-ssm';

interface CertProps extends cdk.StackProps {
    zoneName: string;
    zoneId: string;
    apiDomain: string;
    paramName: string;
}

export class CertStack extends cdk.Stack {
  zone: route53.IHostedZone;
  cert: acm.ICertificate;
  domainName: string;

  constructor(scope: Construct, id: string, props: CertProps) {
    super(scope, id, props);

    this.domainName = `${props.apiDomain}.${props.zoneName}`;

    this.zone = route53.HostedZone.fromHostedZoneAttributes(this, 'Zone', {
        hostedZoneId: props.zoneId,
        zoneName: props.zoneName,
    });

    this.cert = new acm.Certificate(this, 'Cert', {
        domainName: this.domainName,
        validation: acm.CertificateValidation.fromDns(this.zone),
        transparencyLoggingEnabled: false,
    });

    new ssm.StringParameter(this, 'CertArnParam', {
        parameterName: props.paramName,
        stringValue: this.cert.certificateArn,
    });
  }
}
