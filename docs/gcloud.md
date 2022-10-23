List Projects

```
gcloud projects list
```

Set Project

```
gcloud config set project trim-approach-358814
```

# Cloud Run NodeJS Getting Started

https://cloud.google.com/run/docs/quickstarts/build-and-deploy/deploy-nodejs-service

Deploy

```
gcloud run deploy
```

```
This command is equivalent to running `gcloud builds submit --pack image=[IMAGE] /Users/n/code/planet-vector-tile` and `gcloud run deploy planet-vector-tile --image [IMAGE]`
```

```
Building using Buildpacks and deploying container to Cloud Run service [pvtdemo] in project [trim-approach-358814] region [us-central1]
✓ Building and deploying new service... Done.
  ✓ Uploading sources...
  ✓ Building Container... Logs are available at [https://console.cloud.google.com/cloud-build/builds/b0c1bf31-6ce3-4a48-8ee6-b163996894fb?p
  roject=34619323398].
  ✓ Creating Revision... Revision deployment finished. Checking container health.
  ✓ Routing traffic...
  ✓ Setting IAM Policy...
Done.
Service [pvtdemo] revision [pvtdemo-00001-hab] has been deployed and is serving 100 percent of traffic.
Service URL: https://pvtdemo-5vzzysh5ba-uc.a.run.app
```

# NodeJS Job

Dashboard
https://console.cloud.google.com/run/jobs?project=trim-approach-358814

Guide
https://cloud.google.com/run/docs/quickstarts/jobs/build-create-nodejs

Build Container

```
gcloud builds submit --pack image=gcr.io/trim-approach-358814/pvtpoc
```

Create Job

```
gcloud beta run jobs create pvtpoc2 \
    --image gcr.io/trim-approach-358814/pvtpoc \
    --tasks 3 \
    --set-env-vars HELLO=1234 \
    --set-env-vars WORLD=asdf \
    --max-retries 5 \
    --region us-central1
```

Set Default Region

```
gcloud config set run/region us-central1
```

Execute

```
gcloud beta run jobs execute pvtpoc2
```

Run Job Locally

```
docker run --rm -e HELLO=1234 -e WORLD=asdf gcr.io/trim-approach-358814/pvtpoc
```

# Make the right node build be there

```
Error: Cannot find module 'planet-vector-tile-linux-x64-gnu'
```

rustup target add x86_64-unknown-linux-gnu

# Composite many Cloud Storage objects

Combine many objects into 1

https://cloud.google.com/storage/docs/composite-objects

https://cloud.google.com/storage/docs/gsutil/commands/compose

https://stackoverflow.com/questions/43024522/how-to-concatenate-a-million-files-in-google-cloud-storage

# Download byte range
https://github.com/googleapis/nodejs-storage/blob/main/samples/downloadByteRange.js

