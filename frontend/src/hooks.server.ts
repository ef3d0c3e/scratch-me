import type { Handle, HandleFetch } from '@sveltejs/kit/hooks';
import { parseLang } from '#lib/i18n/index.svelte.ts';
import { parseSkin } from '#lib/theme.ts';

/** Where the server reaches the API. The browser goes through /api instead. */
const API_URL = process.env.API_URL ?? 'http://127.0.0.1:8081';

export const handle: Handle = async ({ event, resolve }) => {
	// The theme is in the HTML from the first byte: no flash of the default one
	event.locals.skin = parseSkin(event.cookies.get('skin'));
	event.locals.lang =
		parseLang(event.cookies.get('lang')) ??
		parseLang(event.request.headers.get('accept-language')?.split(',')[0]) ??
		'en';
	return resolve(event, {
		transformPageChunk: ({ html }) =>
			html.replace('%lang%', event.locals.lang).replace('%skin%', event.locals.skin)
	});
};

/** Server-side fetches of /api go straight to the API, with the visitor's cookies */
export const handleFetch: HandleFetch = async ({ event, request, fetch }) => {
	const url = new URL(request.url);
	if (url.origin === event.url.origin && url.pathname.startsWith('/api/')) {
		const target = new URL(url.pathname + url.search, API_URL);
		const headers = new Headers(request.headers);
		const cookie = event.request.headers.get('cookie');
		if (cookie) headers.set('cookie', cookie);
		request = new Request(target, {
			method: request.method,
			headers,
			body: request.body,
			// @ts-expect-error duplex is required by Node for streamed bodies
			duplex: 'half'
		});
	}
	return fetch(request);
};
