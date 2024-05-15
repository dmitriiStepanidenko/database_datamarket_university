let access_control = "env:PUBLIC_BACKEND";
access_control = `${access_control ? access_control : 'same-site'}`;

/** @type {import('houdini').ConfigFile} */
const config = {
  "watchSchema": {
    "url": 'env:PUBLIC_BACKEND',
    "headers": {
      'access-control-allow-origin': access_control,
    }
  },
  "plugins": {
    "houdini-svelte": {}
  },
  "features": {
    "imperativeCache": true
  },
  "scalars": {
    "Thing": {
      "type": "string"
    }
  }
}

export default config
