/**
 * The logged-in player, shared by every page of the app through context:
 * actions update it from their answers, and the ticket being scratched lives
 * here so any page can open one.
 */

import { getContext, setContext } from 'svelte';
import { api } from './api.ts';
import type { Cosmetic, MeDto, Scratched, TicketDto } from './types.ts';

export class Session {
	me = $state() as MeDto;
	/** The ticket on screen, if any */
	open = $state<TicketDto | null>(null);

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
		return this.me.owned.some((o) => o.kind === item.kind && o.id === item.id);
	}

	/** A ticket was scratched: it leaves the stack, its prize lands */
	spent(r: Scratched) {
		this.me.coins = r.coins;
		this.me.tickets = this.me.tickets.filter((t) => t.id !== r.id);
		this.add(r.tickets);
		if (r.prize.kind === 'cosmetic' && !r.duplicate)
			this.me.owned = [...this.me.owned, r.prize.item];
		if (r.prize.kind === 'pocket_refill') void this.refresh().catch(() => {});
	}

	/** Opens the next ticket in the stack, or a given one */
	scratch(ticket: TicketDto | null = this.me.tickets[0] ?? null) {
		this.open = ticket;
	}
}

const KEY = Symbol('session');
export const setSession = (me: MeDto) => setContext(KEY, new Session(me));
export const getSession = () => getContext<Session>(KEY);
