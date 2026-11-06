<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import type { SnowballLayout } from '#lib/types.ts';

	/** The bumps are under the foil; once they show, the snowball rolls down and grows */
	interface Props {
		layout: SnowballLayout | null;
		revealed: boolean;
		foil: Snippet;
		onsettled: () => void;
		ontick?: () => void;
	}
	let { layout, revealed, foil, onsettled, ontick }: Props = $props();
	const i18n = getI18n();
	let step = $state(-1);
	let started = false;
	$effect(() => {
		if (!revealed || !layout || started) return;
		started = true;
		const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;
		const each = reduced ? 0 : 600;
		layout.bumps.forEach((_, i) =>
			setTimeout(
				() => {
					step = i;
					ontick?.();
				},
				300 + i * each
			)
		);
		setTimeout(onsettled, 300 + layout.bumps.length * each + 200);
	});
	const size = $derived(step < 0 || !layout ? 1 : layout.sizes[step]);
</script>

<p class="rules">{i18n.t('series.snowball.rules')}</p>
<div class="play strip">
	<div class="bumps">
		{#each { length: 5 }, i (i)}
			{@const b = layout?.bumps[i]}
			<span class="bump" class:rock={b !== undefined && b < 0} class:now={i === step}>
				{#if b !== undefined}{b < 0 ? '🪨' : '❄️'}<small>{b > 0 ? `+${b}` : b}</small>{/if}
			</span>
		{/each}
	</div>
	{@render foil()}
</div>
<div class="hill" aria-hidden="true">
	<span
		class="ball"
		style:--x="{((step + 1) / 6) * 100}%"
		style:--s={Math.min(size, 14)}
		data-testid="snowball"
		data-size={size}
	></span>
</div>
{#if step >= 0 && layout}
	<p class="size">{i18n.t('snow.size', { n: size })}</p>
{/if}
<p class="pays">{i18n.t('snow.pays')}</p>

<style>
	.strip {
		aspect-ratio: 5 / 1.4 !important;
	}
	.bumps {
		height: 100%;
		display: grid;
		grid-template-columns: repeat(5, 1fr);
		gap: 6px;
		padding: 8px;
	}
	.bump {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		border-radius: 10px;
		background: #e0f2fe;
		font-size: 26px;
	}
	.bump small {
		font: 400 14px/1 var(--display);
		color: #0369a1;
	}
	.bump.rock {
		background: #e5e7eb;
	}
	.bump.now {
		box-shadow: 0 0 0 3px #0ea5e9;
	}
	.hill {
		position: relative;
		height: 120px;
		margin-top: 12px;
		border-radius: 14px;
		background: linear-gradient(170deg, #0c4a6e 0 30%, #f0f9ff 30.5%);
		overflow: hidden;
	}
	.ball {
		position: absolute;
		left: calc(var(--x) - 10px);
		top: calc(30% + var(--x) * 0.25 - var(--s) * 3px);
		width: calc(10px + var(--s) * 5px);
		height: calc(10px + var(--s) * 5px);
		border-radius: 50%;
		background: radial-gradient(circle at 35% 30%, #fff, #bae6fd);
		box-shadow: 0 4px 8px rgba(3, 105, 161, 0.4);
		transition:
			left 0.55s ease-in,
			top 0.55s ease-in,
			width 0.55s ease,
			height 0.55s ease;
	}
	.size {
		text-align: center;
		font: 400 20px/1 var(--display);
		color: #0369a1;
		margin-top: 10px;
	}
	.pays {
		text-align: center;
		font-size: 12px;
		font-weight: 700;
		margin-top: 6px;
	}
</style>
