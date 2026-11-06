<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import type { BoardWalkLayout } from '#lib/types.ts';

	/** The rolls are under the foil; once they show, the token walks the board */
	interface Props {
		layout: BoardWalkLayout | null;
		revealed: boolean;
		foil: Snippet;
		onsettled: () => void;
		ontick?: () => void;
	}
	let { layout, revealed, foil, onsettled, ontick }: Props = $props();
	const i18n = getI18n();
	/** The board, the same on every ticket */
	const BOARD = [0, 5, 0, 10, 0, 0, 20, 5, 0, 40, 0, 0, 10, 5, 0, 120];
	/** Squares round a 5×5 grid, clockwise from the bottom left */
	const SPOTS: [number, number][] = [
		[4, 0],
		[3, 0],
		[2, 0],
		[1, 0],
		[0, 0],
		[0, 1],
		[0, 2],
		[0, 3],
		[0, 4],
		[1, 4],
		[2, 4],
		[3, 4],
		[4, 4],
		[4, 3],
		[4, 2],
		[4, 1]
	];
	let at = $state(0);
	let landed = $state<number[]>([]);
	let started = false;
	$effect(() => {
		if (!revealed || !layout || started) return;
		started = true;
		if (matchMedia('(prefers-reduced-motion: reduce)').matches) {
			at = layout.path.at(-1) ?? 0;
			landed = layout.path;
			setTimeout(onsettled, 100);
			return;
		}
		let t = 400;
		let pos = 0;
		layout.rolls.forEach((r, k) => {
			for (let s = 0; s < r; s++) {
				pos = (pos + 1) % 16;
				const p = pos;
				setTimeout(() => {
					at = p;
					ontick?.();
				}, t);
				t += 220;
			}
			const stop = layout.path[k];
			setTimeout(() => (landed = [...landed, stop]), t);
			t += 350;
		});
		setTimeout(onsettled, t + 100);
	});
</script>

<p class="rules">{i18n.t('series.board_walk.rules')}</p>
<div class="play strip">
	<div class="dice">
		{#each { length: 3 }, i (i)}
			<span class="die">{layout ? ['⚀', '⚁', '⚂', '⚃', '⚄', '⚅'][layout.rolls[i] - 1] : ''}</span>
		{/each}
	</div>
	{@render foil()}
</div>
<div class="board" aria-hidden="true">
	{#each BOARD as v, i (i)}
		{@const [r, c] = SPOTS[i]}
		<span
			class="sq"
			class:start={i === 0}
			class:big={v >= 100}
			class:landed={landed.includes(i)}
			style:grid-row={r + 1}
			style:grid-column={c + 1}
			data-square={i}
			data-landed={landed.includes(i) ? '' : undefined}>{i === 0 ? 'GO' : v ? v : ''}</span
		>
	{/each}
	<span class="token" style:grid-row={SPOTS[at][0] + 1} style:grid-column={SPOTS[at][1] + 1}
		>🎩</span
	>
	<span class="middle">{i18n.t('series.board_walk')}</span>
</div>

<style>
	.strip {
		aspect-ratio: 3 / 1.1 !important;
	}
	.dice {
		height: 100%;
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		place-items: center;
	}
	.die {
		font-size: 64px;
		line-height: 1;
		color: #a16207;
	}
	.board {
		position: relative;
		display: grid;
		grid-template-columns: repeat(5, 1fr);
		grid-template-rows: repeat(5, 1fr);
		gap: 4px;
		aspect-ratio: 1;
		width: min(280px, 76vw);
		margin: 12px auto 0;
	}
	.sq {
		display: grid;
		place-items: center;
		border-radius: 8px;
		background: #fef3c7;
		border: 2px solid #eab308;
		font: 400 14px/1 var(--display);
		color: #a16207;
	}
	.sq.start {
		background: #eab308;
		color: #fff;
	}
	.sq.big {
		background: #fde047;
	}
	.sq.landed {
		background: #22c55e;
		color: #fff;
		animation: pop 0.5s cubic-bezier(0.3, 1.6, 0.5, 1) both;
	}
	.token {
		display: grid;
		place-items: center;
		font-size: 26px;
		z-index: 1;
		pointer-events: none;
		filter: drop-shadow(0 2px 2px rgba(0, 0, 0, 0.4));
	}
	.middle {
		grid-row: 2 / 5;
		grid-column: 2 / 5;
		display: grid;
		place-items: center;
		font: 400 18px/1 var(--display);
		color: #a16207;
		opacity: 0.5;
	}
	@keyframes pop {
		40% {
			transform: scale(1.2);
		}
	}
</style>
