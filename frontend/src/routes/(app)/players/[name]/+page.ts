import { api } from '#lib/api.ts';
import type { AchievementDto, ProfileDto } from '#lib/types.ts';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, params }) => {
	const [profile, mine] = await Promise.all([
		api<ProfileDto>(`/players/${encodeURIComponent(params.name)}`, {}, fetch),
		api<AchievementDto[]>('/achievements', {}, fetch)
	]);
	return { profile, mine };
};
