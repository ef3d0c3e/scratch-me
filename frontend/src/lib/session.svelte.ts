/**
 * The logged-in player, shared by every page of the app through context:
 * actions update it from their answers, and the ticket being scratched lives
 * here so any page can open one.
 */

import { getContext, setContext } from 'svelte';
import { api } from './api.ts';
import type { Cosmetic, MeDto, Prize, Progress, Scratched, TicketDto } from './types.ts';

/** Something to tell the player, shown a few seconds */
export interface News {
	id: number;
	kind: 'level' | 'quest' | 'bonus' | 'achievement';
	what: string;
	reward: Prize;
}

export class Session {
	me = $state() as MeDto;
	/** The ticket on screen, if any */
	open = $state<TicketDto | null>(null);
	/** Level ups, quests and achievements waiting to be shown */
	news = $state<News[]>([]);
	/** Moves on each time progress lands: pages showing quests refetch them */
	progressed = $state(0);
	private newsId = 0;

	constructor(me: MeDto) {
		this.me = me;
	}

	async refresh() {
		this.me = await api<MeDto>('/me');
	}

	add(tickets: TicketDto[]) {
		this.me.tickets = [...this.me.tickets, ...tickets];
	}

	owns(item: Cosmetic) {
		if (item.kind === 'theme' && item.id === 'midnight') return true;
		if (item.kind === 'tool' && item.id === 'coin') return true;
		if (item.kind === 'foil' && item.id === 'silver') return true;
		return this.me.owned.some((o) => o.kind === item.kind && o.id === item.id);
	}

	/** What an action's progress gave: balance, level, reward tickets, and the news */
	apply(p: Progress) {
		this.me.coins = p.coins;
		this.me.level = p.level;
		this.add(p.tickets);
		const push = (kind: News['kind'], what: string, reward: Prize) =>
			this.news.push({ id: ++this.newsId, kind, what, reward });
		for (const l of p.level_ups) push('level', l.what, l.reward);
		for (const q of p.quests) push('quest', q.what, q.reward);
		if (p.bonus) push('bonus', '', p.bonus);
		for (const a of p.achievements) push('achievement', a.what, a.reward);
		this.progressed++;
	}

	dismiss(id: number) {
		this.news = this.news.filter((n) => n.id !== id);
	}

	/** A ticket was scratched: it leaves the stack, its prize lands */
	spent(r: Scratched) {
		this.me.tickets = this.me.tickets.filter((t) => t.id !== r.id);
		this.add(r.tickets);
		this.apply(r.progress);
		if (r.prize.kind === 'cosmetic' && !r.duplicate)
			this.me.owned = [...this.me.owned, r.prize.item];
		if (r.prize.kind === 'pocket_refill') void this.refresh().catch(() => {});
	}

	/** A ticket the stack still showed but that is gone: out, and the rest refreshed */
	forget(id: number) {
		this.me.tickets = this.me.tickets.filter((t) => t.id !== id);
		void this.refresh().catch(() => {});
	}

	/** Opens the next ticket in the stack, or a given one */
	scratch(ticket: TicketDto | null = this.me.tickets[0] ?? null) {
		this.open = ticket;
	}
}

const KEY = Symbol('session');
export const setSession = (me: MeDto) => setContext(KEY, new Session(me));
export const getSession = () => getContext<Session>(KEY);
