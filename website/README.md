# Website

This website is built using [Docusaurus](https://docusaurus.io/), a modern static website generator.

## Installation

```bash
yarn
```

## Local Development

```bash
npm run dev
```

This command starts the Docusaurus hot-reload development server for one locale.

To preview language switching locally, use:

```bash
npm run start
```

`npm run start` builds all locales and serves the production build, so the locale dropdown routes (`/es`, `/zh-CN`) resolve the same way they do on the deployed site.

## Build

```bash
npm run build
```

This command generates static content into the `build` directory and can be served using any static contents hosting service.

## Deployment

Using SSH:

```bash
USE_SSH=true yarn deploy
```

Not using SSH:

```bash
GIT_USER=<Your GitHub username> yarn deploy
```

If you are using GitHub pages for hosting, this command is a convenient way to build the website and push to the `gh-pages` branch.
