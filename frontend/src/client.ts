import { HoudiniClient } from '$houdini';
import { env } from '$env/dynamic/public';

const url = `${env.PUBLIC_BACKEND ? env.PUBLIC_BACKEND : 'http://localhost:55000'}`;

export default new HoudiniClient({
  url,
  fetchParams() {
    return {
      headers: {
        'Sec-Fetch-Site': 'same-site',
        //'Access-Control-Allow-Origin': access_control,
      },
      variables: {
        credentials: 'include'
      },
      credentials: 'include',
    }
  }

  // uncomment this to configure the network call (for things like authentication)
  // for more information, please visit here: https://www.houdinigraphql.com/guides/authentication
  // fetchParams({ session }) {
  //     return {
  //         headers: {
  //             Authentication: `Bearer ${session.token}`,
  //         }
  //     }
  // }
})
