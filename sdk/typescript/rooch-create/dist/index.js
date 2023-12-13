#!/usr/bin/env node
"use strict";var t=require("create-create-app"),r=require("path");var e={name:"create-rooch",version:"0.0.6",description:"Create a new Rooch project",license:"Apache-2.0",author:"Rooch.network <opensource@rooch.network>",bin:{rooch:"dist/index.js"},files:["dist"],scripts:{build:"pnpm run build:js","build:js":"tsup && ./scripts/copy-templates.sh",clean:"pnpm run clean:js","clean:js":"rimraf dist",dev:"tsup --watch",prepublishOnly:"npm run clean && npm run build",test:"pnpm run test:react","test:ci":"pnpm run test","test:react":"dist/cli.js test-project --template react && rimraf test-project"},dependencies:{"create-create-app":"git+https://github.com/holic/create-create-app#74376c59b48a04aabbe94d9cacfe9cb1cecccd63"},devDependencies:{"@types/node":"^18.15.13",tsup:"^6.7.0"},publishConfig:{access:"public",registry:"https://registry.npmjs.org"}};var a=(0,r.resolve)(__dirname,"..","dist","templates");(0,t.create)("create-rooch",{templateRoot:a,defaultTemplate:"react",defaultPackageManager:"pnpm",promptForDescription:!1,promptForAuthor:!1,promptForEmail:!1,promptForLicense:!1,promptForTemplate:!0,caveat:({answers:o,packageManager:s})=>`Done! Play in the rooch with \`cd ${o.name}\` and \`${s} run dev\``,extra:{"rooch-version":{type:"input",describe:"The version of Rooch packages to use, defaults to latest",default:e.version}}});