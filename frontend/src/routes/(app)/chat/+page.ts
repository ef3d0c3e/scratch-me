import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';

// Guests have no chat
export const load: PageLoad = async ({ parent }) => {
	const { me } = await parent();
	if (me.guest) redirect(303, '/');
};
