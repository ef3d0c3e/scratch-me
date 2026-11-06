<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import type { PlinkoLayout } from '#lib/types.ts';

	/** The foil hides how many balls; once revealed they drop, one after the other */
	interface Props {
		layout: PlinkoLayout | null;
		revealed: boolean;
		foil: Snippet;
		onsettled: () => void;
		ontick?: () => void;
	}
	let { layout, revealed, foil, onsettled, ontick }: Props = $props();
	const i18n = getI18n();
	const ROWS = 8;
	const W = 300;
	const H = 300;
	const GAP = W / (ROWS + 2);
	const STEP = 200;
	const BIN_COUNT = ROWS + 1;

	/** Each ball's position: row reached and rights so far */
	let balls = $state<{ row: number; rights: number; landed: boolean }[]>([]);
	let started = false;

	$effect(() => {
		if (!revealed || !layout || started) return;
		started = true;
		const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;
		const drops = layout.balls;
		balls = drops.map(() => ({ row: -1, rights: 0, landed: false }));
		if (reduced) {
			balls = drops.map((d) => ({ row: ROWS, rights: d.bin, landed: true }));
			setTimeout(onsettled, 100);
			return;
		}
		let t = 300;
		drops.forEach((d, b) => {
			for (let r = 0; r <= ROWS; r++) {
				setTimeout(() => {
					const rights = d.path.slice(0, r).filter(Boolean).length;
					balls[b] = { row: r, rights, landed: r === ROWS };
					ontick?.();
				}, t);
				t += STEP;
			}
			t += 250;
		});
		setTimeout(onsettled, t + 200);
	});

	const x = (row: number, rights: number) => W / 2 + (rights - row / 2) * GAP;
	const y = (row: number) => 30 + (row + 0.5) * ((H - 70) / ROWS);
	const binX = (i: number) => W / 2 + (i - ROWS / 2) * GAP;
	const landedIn = (i: number) => balls.some((b, k) => b.landed && layout?.balls[k].bin === i);
</script>

<p class="rules">{i18n.t('series.plinko.rules')}</p>
<div class="play spot">
	<div class="count">
		{#if layout}
			<span aria-hidden="true">{'⚪'.repeat(layout.balls.length)}</span>
			<b data-testid="plinko-balls">{i18n.t('plinko.balls', { n: layout.balls.length })}</b>
		{/if}
	</div>
	{@render foil()}
</div>
<svg class="board" viewBox="0 0 {W} {H}" aria-hidden="true">
	{#each { length: ROWS }, r (r)}
		{#each { length: r + 1 }, k (k)}
			<circle cx={x(r, k)} cy={y(r) - 8} r="3.2" class="peg" />
		{/each}
	{/each}
	{#each layout?.bins ?? [] as v, i (i)}
		<g class="bin" class:hit={landedIn(i)}>
			<rect x={binX(i) - GAP / 2 + 2} y={H - 38} width={GAP - 4} height="34" rx="6" />
			<text x={binX(i)} y={H - 16} text-anchor="middle">{v >= 1000 ? `${v / 1000}k` : v}</text>
		</g>
	{/each}
	{#each balls as b, i (i)}
		{#if b.row >= 0}
			<circle
				class="ball"
				cx={b.landed ? binX(layout?.balls[i].bin ?? 0) : x(b.row, b.rights)}
				cy={b.landed ? H - 46 : y(b.row) - 18}
				r="7"
			/>
		{/if}
	{/each}
</svg>

<style>
	.count {
		height: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 10px;
		border-radius: 14px;
		background: var(--paper);
		border: 2px dashed #e8b4f0;
	}
	.count b {
		font: 400 22px/1 var(--display);
		color: #a21caf;
	}
	.board {
		display: block;
		width: min(300px, 80vw);
		margin: 10px auto 0;
		border-radius: 14px;
		background: linear-gradient(#2b1a5c, #1f1245);
	}
	.peg {
		fill: #e8b4f0;
	}
	.bin rect {
		fill: #3d2470;
	}
	.bin text {
		fill: #fff6e5;
		font: 700 11px var(--body);
	}
	.bin.hit rect {
		fill: #ffc23d;
	}
	.bin.hit text {
		fill: #3a1d00;
	}
	.ball {
		fill: #fff;
		stroke: #d946ef;
		stroke-width: 2;
		transition:
			cx 0.18s ease-in,
			cy 0.18s ease-in;
	}
</style>
