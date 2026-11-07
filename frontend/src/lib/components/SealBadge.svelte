<script lang="ts">
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { SEAL_LOOK, sealEffects } from '#lib/seals.ts';
	import type { Seal } from '#lib/types.ts';

	/** A wax seal: the bundle's icon, and what it does to the prize */
	let { seal, small = false }: { seal: Seal; small?: boolean } = $props();
	const i18n = getI18n();
	const look = $derived(SEAL_LOOK[seal.theme]);
	const label = $derived(
		[i18n.t(`seal.${seal.key}` as never), ...sealEffects(i18n, seal)].join(' · ')
	);
	const short = $derived(
		seal.times > 1
			? `×${seal.times}`
			: seal.sure
				? '✓'
				: seal.best > 1
					? `${seal.best}↑`
					: seal.insurance > 0
						? `${seal.insurance}%`
						: seal.echo
							? '+1'
							: '↻'
	);
</script>

<span
	class="seal"
	class:small
	style:--wax={look.wax}
	title={label}
	aria-label={label}
	role="img"
	data-seal={seal.key}
>
	<span class="icon" aria-hidden="true">{look.icon}</span>
	<b aria-hidden="true">{short}</b>
</span>

<style>
	.seal {
		position: relative;
		display: inline-grid;
		place-items: center;
		width: 46px;
		height: 46px;
		border-radius: 50%;
		background: radial-gradient(
			circle at 35% 30%,
			color-mix(in srgb, var(--wax) 60%, #fff),
			var(--wax) 60%
		);
		box-shadow:
			0 0 0 3px color-mix(in srgb, var(--wax) 70%, #000),
			0 4px 10px -2px rgba(0, 0, 0, 0.5);
		transform: rotate(-12deg);
	}
	.icon {
		font-size: 20px;
		line-height: 1;
	}
	b {
		position: absolute;
		bottom: -6px;
		right: -8px;
		padding: 1px 5px;
		border-radius: 999px;
		background: #1b1530;
		color: #ffd66b;
		font: 400 12px/1.2 var(--display);
		transform: rotate(12deg);
	}
	.small {
		width: 30px;
		height: 30px;
	}
	.small .icon {
		font-size: 14px;
	}
	.small b {
		font-size: 10px;
		bottom: -5px;
		right: -7px;
	}
</style>
