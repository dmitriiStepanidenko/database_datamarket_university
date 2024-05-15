import { setSession } from '$houdini'
import type { Handle } from '@sveltejs/kit'

export const handle: Handle = async ({ event, resolve }) => {
    // get the user information however you want
    //const user = await authenticateUser(event)

    //// set the session information for this event
    //setSession(event, { user })

    // pass the event onto the default handle
    return await resolve(event)
}
