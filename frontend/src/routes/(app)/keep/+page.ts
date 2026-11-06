import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';

// Only guests have progress to keep
export const load: PageLoad = async ({ parent }) => {
	const { me } = await parent();
	if (!me.guest) redirect(303, '/');
};
