import { api } from '#lib/api.ts';
import type { Board, LeaderboardDto } from '#lib/types.ts';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, url }) => {
	const by: Board = url.searchParams.get('by') === 'level' ? 'level' : 'coins';
	return { board: await api<LeaderboardDto>(`/leaderboard?by=${by}`, {}, fetch) };
};
