import { redirect } from '@sveltejs/kit';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ fetch }) => {
	// Already logged in: straight to the lobby
	const me = await fetch('/api/me').catch(() => null);
	if (me?.ok) redirect(303, '/');
};
