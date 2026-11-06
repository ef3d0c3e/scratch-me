import { api } from '#lib/api.ts';
import type { AchievementDto } from '#lib/types.ts';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => ({
	achievements: await api<AchievementDto[]>('/achievements', {}, fetch)
});
