import { describe, expect, it } from 'vitest';
import { Live, MAX_MESSAGES } from './live.svelte.ts';

const shout = (id: number, user = 'ana') => ({
	id,
	user,
	level: 1,
	title: null,
	body: `hi ${id}`,
	at: '2026-11-06T10:00:00Z'
});
const win = (user: string) => ({
	type: 'win' as const,
	user,
	series: 'lucky_three' as const,
	prize: { kind: 'coins' as const, amount: 1000 },
	at: '2026-11-06T10:00:00Z'
});

describe('Live', () => {
	it('takes the snapshot, then adds shouts and counts the unread', () => {
		const live = new Live();
		live.receive({ type: 'hello', messages: [shout(1)], feed: [], announcements: [] });
		live.receive({ type: 'chat', ...shout(2) });
		expect(live.messages.map((m) => m.id)).toEqual([1, 2]);
		expect(live.unread).toBe(1);
		live.chatOpen = true;
		live.receive({ type: 'chat', ...shout(3) });
		expect(live.unread).toBe(1);
	});

	it('does not show a shout twice: once from the POST, once from the socket', () => {
		const live = new Live();
		live.posted(shout(5));
		live.receive({ type: 'chat', ...shout(5) });
		expect(live.messages).toHaveLength(1);
	});

	it('keeps a bounded history and leaves the muted out', () => {
		const live = new Live();
		live.muted = new Set(['troll']);
		for (let i = 0; i < MAX_MESSAGES + 20; i++) live.receive({ type: 'chat', ...shout(i) });
		live.receive({ type: 'chat', ...shout(999, 'troll') });
		expect(live.messages).toHaveLength(MAX_MESSAGES);
		expect(live.messages.some((m) => m.user === 'troll')).toBe(false);
	});

	it('holds our own wins while a ticket is on screen, not the others', () => {
		const live = new Live();
		live.hold('me');
		live.receive(win('me'));
		live.receive(win('bob'));
		expect(live.feed.map((f) => f.user)).toEqual(['bob']);
		live.release();
		expect(live.feed.map((f) => f.user)).toEqual(['me', 'bob']);
	});

	it('adds and takes down announcements', () => {
		const live = new Live();
		live.receive({ type: 'announcement', id: 1, body: 'Happy hour', at: '2026-11-06T10:00:00Z' });
		live.receive({ type: 'announcement', id: 2, body: 'New ticket', at: '2026-11-06T11:00:00Z' });
		expect(live.announcements.map((a) => a.id)).toEqual([2, 1]);
		live.receive({ type: 'unannounced', id: 1 });
		expect(live.announcements.map((a) => a.id)).toEqual([2]);
	});
});
