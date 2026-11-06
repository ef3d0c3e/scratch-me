/**
 * The live socket: shouts, big wins, achievements, announcements. Push only:
 * sending goes through the REST API. Reconnects on its own after a drop.
 * Taken from AnyCardz.
 */

import { getContext, setContext } from 'svelte';
import { api } from './api.ts';
import type { AnnouncementDto, ChatEvent, DuelStatus, FeedItem, Series } from './types.ts';

type LiveMessage =
	| { type: 'hello'; messages: ChatEvent[]; feed: FeedItem[]; announcements: AnnouncementDto[] }
	| ({ type: 'chat' } & ChatEvent)
	| FeedItem
	| ({ type: 'announcement' } & AnnouncementDto)
	| { type: 'unannounced'; id: number }
	| Notice;

/** Events for one player (or, for the pot, news everyone hears) */
export type Notice =
	| { type: 'gift'; from: string; series: Series }
	| { type: 'duel'; id: number; from: string; status: DuelStatus }
	| { type: 'pot'; user: string; amount: number };

export const MAX_MESSAGES = 100;
export const MAX_FEED = 30;

export class Live {
	messages = $state<ChatEvent[]>([]);
	/** Big wins and achievements, newest first */
	feed = $state<FeedItem[]>([]);
	announcements = $state<AnnouncementDto[]>([]);
	connected = $state(false);
	/** Shouts that came while the chat was not on screen */
	unread = $state(0);
	chatOpen = $state(false);
	/** Players muted: their shouts are left out */
	muted = $state<Set<string>>(new Set());
	/** Moves on with each duel or pot news: cards showing them refetch */
	social = $state(0);
	/** Told of gifts, duels and pots */
	onnotice: ((n: Notice) => void) | null = null;

	#ws: WebSocket | null = null;
	#retry = 1000;
	#closed = false;
	#timer: ReturnType<typeof setTimeout> | undefined;
	/** While a ticket is on screen: the player's own news waits, it would spoil the reveal */
	#holdFor: string | null = null;
	#held: LiveMessage[] = [];

	/** `chat`: false for guests, who have none (the server sends them no shouts) */
	start({ chat = true } = {}) {
		this.#closed = false;
		this.#connect();
		if (chat) void this.loadMutes();
	}

	stop() {
		this.#closed = true;
		clearTimeout(this.#timer);
		this.#ws?.close();
	}

	async loadMutes() {
		const list = await api<string[]>('/mutes').catch(() => null);
		if (!list) return;
		this.muted = new Set(list);
		this.messages = this.messages.filter((m) => !this.muted.has(m.user));
	}

	#connect() {
		const proto = location.protocol === 'https:' ? 'wss' : 'ws';
		const ws = new WebSocket(`${proto}://${location.host}/api/live`);
		this.#ws = ws;
		// A socket replaced since (a restart) has nothing more to say
		ws.onopen = () => {
			if (ws !== this.#ws) return;
			this.connected = true;
			this.#retry = 1000;
		};
		ws.onmessage = (e) => {
			if (ws === this.#ws) this.receive(JSON.parse(e.data) as LiveMessage);
		};
		ws.onclose = () => {
			if (ws !== this.#ws) return;
			this.connected = false;
			if (this.#closed) return;
			// Back off up to 30 s, so a server restart is not hammered
			this.#timer = setTimeout(() => this.#connect(), this.#retry);
			this.#retry = Math.min(this.#retry * 2, 30_000);
		};
	}

	/** Holds the player's own wins and achievements until `release` */
	hold(username: string) {
		this.#holdFor = username;
	}

	release() {
		this.#holdFor = null;
		const held = this.#held;
		this.#held = [];
		for (const m of held) this.receive(m);
	}

	/** Handles one message from the socket (public for tests) */
	receive(m: LiveMessage) {
		if (
			this.#holdFor !== null &&
			(m.type === 'win' || m.type === 'feat') &&
			m.user === this.#holdFor
		) {
			this.#held.push(m);
			return;
		}
		switch (m.type) {
			case 'hello':
				this.messages = m.messages.filter((x) => !this.muted.has(x.user));
				this.feed = m.feed;
				this.announcements = m.announcements;
				break;
			case 'chat': {
				if (this.muted.has(m.user)) break;
				const { type: _, ...shout } = m;
				// Our own shout may already be there, from the POST's answer
				if (this.messages.some((x) => x.id === shout.id)) break;
				this.messages = [...this.messages, shout].slice(-MAX_MESSAGES);
				if (!this.chatOpen) this.unread++;
				break;
			}
			case 'win':
			case 'feat':
				this.feed = [m, ...this.feed].slice(0, MAX_FEED);
				break;
			case 'announcement': {
				const { type: _, ...a } = m;
				this.announcements = [a, ...this.announcements.filter((x) => x.id !== a.id)];
				break;
			}
			case 'unannounced':
				this.announcements = this.announcements.filter((x) => x.id !== m.id);
				break;
			case 'gift':
			case 'duel':
			case 'pot':
				this.social++;
				this.onnotice?.(m);
				break;
		}
	}

	/** A shout we just posted, shown at once */
	posted(shout: ChatEvent) {
		if (!this.messages.some((x) => x.id === shout.id))
			this.messages = [...this.messages, shout].slice(-MAX_MESSAGES);
	}
}

const KEY = Symbol('live');
export const setLive = () => setContext(KEY, new Live());
export const getLive = () => getContext<Live>(KEY);
