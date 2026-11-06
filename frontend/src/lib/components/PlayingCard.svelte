<script lang="ts">
	import type { PlayingCard } from '#lib/types.ts';
	/** A playing card face: rank and suit, red or black */
	let { card, big = false }: { card: PlayingCard; big?: boolean } = $props();
	const RANKS: Record<number, string> = { 11: 'J', 12: 'Q', 13: 'K', 14: 'A' };
	const SUITS = { spades: '♠', hearts: '♥', diamonds: '♦', clubs: '♣' };
	const rank = $derived(RANKS[card.rank] ?? String(card.rank));
	const red = $derived(card.suit === 'hearts' || card.suit === 'diamonds');
</script>

<span class="card" class:big class:red role="img" aria-label="{rank} {card.suit}">
	<b>{rank}</b><i>{SUITS[card.suit]}</i>
</span>

<style>
	.card {
		display: inline-flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		width: 56px;
		height: 78px;
		border-radius: 8px;
		background: #fff;
		color: #1b1530;
		box-shadow:
			0 0 0 1.5px #d9d1bf,
			0 3px 0 #d9d1bf;
		font-family: var(--display);
		line-height: 1;
	}
	.card.big {
		width: 74px;
		height: 102px;
	}
	.card.red {
		color: #d61f3b;
	}
	b {
		font-weight: 400;
		font-size: 24px;
	}
	.big b {
		font-size: 32px;
	}
	i {
		font-style: normal;
		font-size: 22px;
	}
</style>
