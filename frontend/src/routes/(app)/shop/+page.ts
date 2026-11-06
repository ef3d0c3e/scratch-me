import { api } from '#lib/api.ts';
import type { ShopDto } from '#lib/types.ts';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => ({
	shop: await api<ShopDto>('/shop', {}, fetch)
});
