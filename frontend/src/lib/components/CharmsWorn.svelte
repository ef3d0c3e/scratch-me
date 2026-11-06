<script lang="ts">
	import { onMount } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { getSession } from '#lib/session.svelte.ts';
	import { CHARM_ICON, going } from '#lib/charms.ts';
	import { countdown } from '#lib/time.ts';

	const i18n = getI18n();
	const session = getSession();
	let now = $state(Date.now());
	onMount(() => {
		const id = setInterval(() => (now = Date.now()), 1000);
		return () => clearInterval(id);
	});
	const worn = $derived(going(session.me.charms, now));
</script>

{#if worn.length}
	<section class="charms" aria-labelledby="charms-title">
		<h2 id="charms-title">{i18n.t('charm.worn_title')}</h2>
		<ul>
			{#each worn as c (c.kind)}
				<li class="chip {c.tier}" data-charm={c.kind}>
					<span class="icon" aria-hidden="true">{CHARM_ICON[c.kind]}</span>
					<span class="name">
						{i18n.t(`charm.tier.${c.tier}`)}
						{i18n.t(`charm.${c.kind}`)}
						<small>{i18n.t(`charm.${c.kind}.effect`, { n: c.strength })}</small>
					</span>
					<span class="left">
						<b class="num">{i18n.t('charm.left', { n: c.charges })}</b>
						<span class="num">⏳ {countdown(c.expires_at, now)}</span>
					</span>
				</li>
			{/each}
		</ul>
	</section>
{/if}

<style>
	h2 {
		font-size: 14px;
		color: var(--muted);
		margin-bottom: 8px;
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.chip {
		--tier: #cd8a4b;
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px 12px;
		border-radius: 14px;
		background: var(--night-2);
		border: 1px solid var(--tier);
		box-shadow: 0 0 14px -8px var(--tier);
	}
	.chip.silver {
		--tier: #c9d1dc;
	}
	.chip.gold {
		--tier: var(--gold);
	}
	.icon {
		font-size: 22px;
	}
	.name {
		flex: 1;
		font-weight: 700;
		font-size: 14px;
		display: flex;
		flex-direction: column;
	}
	small {
		font-weight: 500;
		color: var(--muted);
		font-size: 12px;
	}
	.left {
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		font-size: 12px;
		color: var(--muted);
	}
	.left b {
		color: var(--tier);
	}
</style>
