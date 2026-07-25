{
  "name": "{{project_name}}",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "start": "node dist/main.js",
    "build": "tsc -p tsconfig.json"
  },
  "dependencies": {
    "@nestjs/common": "^10.0.0",
    "@nestjs/core": "^10.0.0",
    "reflect-metadata": "^0.2.0",
    "rxjs": "^7.8.0"
  }
}
