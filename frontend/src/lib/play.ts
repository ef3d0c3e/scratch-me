import { api, post } from './api.ts';
import type { PlayState } from './types.ts';

/** Where a ticket played move by move stands; a timed one may end on its own */
export const look = <V>(id: number) => api<PlayState<V>>(`/tickets/${id}/play`);

/** One move */
export const move = <V>(id: number, mv: unknown) =>
	post<PlayState<V>>(`/tickets/${id}/play`, { move: mv });
