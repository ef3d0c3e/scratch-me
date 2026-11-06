import { redirect } from '@sveltejs/kit';
import { api, ApiError } from '#lib/api.ts';
import type { MeDto } from '#lib/types.ts';
import type { LayoutLoad } from './$types';

// Universal load: runs on the server for the first page, then in the browser
export const load: LayoutLoad = async ({ fetch }) => {
	try {
		return { me: await api<MeDto>('/me', {}, fetch) };
	} catch (e) {
		if (e instanceof ApiError && e.status === 401) redirect(303, '/login');
		throw e;
	}
};
