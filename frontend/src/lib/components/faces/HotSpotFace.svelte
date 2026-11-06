<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import type { HotSpotLayout } from '#lib/types.ts';

	interface Props {
		layout: HotSpotLayout | null;
		revealed: boolean;
		foil: Snippet;
		onsettled: () => void;
	}
	let { layout, revealed, foil, onsettled }: Props = $props();
	const i18n = getI18n();
	$effect(() => {
		if (revealed && layout) onsettled();
	});
	const hit = (n: number) => revealed && !!layout?.drawn.includes(n) && !!layout?.mine.includes(n);
</script>

<p class="rules">{i18n.t('series.hot_spot.rules')}</p>
<div class="play keno">
	<div class="sheet">
		<p class="label">{i18n.t('hot.mine')}</p>
		<div class="mine">
			{#each { length: 10 }, i (i)}
				{@const n = layout?.mine[i]}
				<span
					class="ball"
					class:hit={n !== undefined && hit(n)}
					data-hit={n !== undefined && hit(n) ? '' : undefined}>{n ?? ''}</span
				>
			{/each}
		</div>
		<p class="label">{i18n.t('hot.drawn')}</p>
		<div class="drawn">
			{#each { length: 20 }, i (i)}
				{@const n = layout?.drawn[i]}
				<span class="small" class:hit={n !== undefined && hit(n)}>{n ?? ''}</span>
			{/each}
		</div>
	</div>
	{@render foil()}
</div>
{#if revealed && layout}
	<p class="count" data-testid="matches">{i18n.t('hot.matches', { n: layout.matches })}</p>
{/if}
<p class="pays">{i18n.t('hot.pays')}</p>

<style>
	.keno {
		aspect-ratio: 1 / 0.85 !important;
	}
	.sheet {
		height: 100%;
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 10px;
	}
	.label {
		font-size: 11px;
		font-weight: 800;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		text-align: center;
		opacity: 0.7;
	}
	.mine {
		display: grid;
		grid-template-columns: repeat(5, 1fr);
		gap: 6px;
	}
	.ball {
		aspect-ratio: 1;
		border-radius: 50%;
		display: grid;
		place-items: center;
		font: 400 18px/1 var(--display);
		background: var(--paper);
		border: 2px solid #f43f5e;
	}
	.ball.hit {
		background: #f43f5e;
		color: #fff;
		animation: pop 0.6s cubic-bezier(0.3, 1.6, 0.5, 1) both;
	}
	.drawn {
		display: grid;
		grid-template-columns: repeat(10, 1fr);
		gap: 3px;
	}
	.small {
		aspect-ratio: 1;
		border-radius: 50%;
		display: grid;
		place-items: center;
		font: 700 11px/1 var(--body);
		background: #fecdd3;
	}
	.small.hit {
		background: #f43f5e;
		color: #fff;
	}
	.count {
		text-align: center;
		font: 400 18px/1 var(--display);
		color: #be123c;
		margin-top: 8px;
	}
	.pays {
		text-align: center;
		font-size: 12px;
		font-weight: 700;
		margin-top: 6px;
	}
	@keyframes pop {
		40% {
			transform: scale(1.2);
		}
	}
</style>
