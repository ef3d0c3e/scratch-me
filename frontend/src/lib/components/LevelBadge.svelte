<script lang="ts">
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import type { LevelDto } from '#lib/types.ts';

	/** The level, with a ring filling toward the next one */
	let { level }: { level: LevelDto } = $props();
	const i18n = getI18n();
	const share = $derived(Math.min(1, (level.xp - level.from) / Math.max(1, level.to - level.from)));
</script>

<span
	class="badge"
	role="img"
	aria-label={i18n.t('level.label', {
		n: level.level,
		xp: level.xp - level.from,
		to: level.to - level.from
	})}
	style:--share={share}
	data-testid="level"
	data-level={level.level}
>
	<span class="n">{level.level}</span>
</span>

<style>
	.badge {
		width: 42px;
		height: 42px;
		border-radius: 50%;
		display: grid;
		place-items: center;
		background: conic-gradient(var(--mint) calc(var(--share) * 360deg), var(--night-3) 0);
		transition: background 0.6s ease;
	}
	.n {
		width: 34px;
		height: 34px;
		border-radius: 50%;
		display: grid;
		place-items: center;
		background: var(--night-2);
		font: 400 15px/1 var(--display);
		color: var(--mint);
	}
</style>
