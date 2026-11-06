<script lang="ts">
	import { onMount } from 'svelte';
	import { Coverage, brush, flakes, ready } from '#lib/scratch/foil.ts';
	import { TOOLS } from '#lib/fx/tools.ts';
	import type { ToolId } from '#lib/types.ts';
	import ToolIcon from './ToolIcon.svelte';
	import {
		IDLE_AFTER,
		approach,
		askTilt,
		drift,
		pointerOffset,
		tiltOffset,
		type Offset
	} from '#lib/fx/tilt.ts';

	/**
	 * Silver foil over whatever sits under it, scratched off with a finger or
	 * the mouse. Flakes fall off as it goes. Once every square of the grid
	 * under it is mostly scratched, the rest goes by itself and `onreveal`
	 * fires once.
	 */
	interface Props {
		/** Set by the parent to clear the foil (reveal all, or reveal at once) */
		revealed: boolean;
		onreveal: () => void;
		/** A finger lands on the foil (a gesture: audio may start here) */
		onscratchstart?: () => void;
		/** The finger moves, at this many px per ms */
		onscratch?: (speed: number) => void;
		onscratchend?: () => void;
		/** The short word printed on the foil; also its accessible name */
		label: string;
		/** The grid under the foil: each square must be scratched before the rest falls off */
		cols?: number;
		/** What the player scratches with: brush, flakes, the icon under the finger */
		tool?: ToolId;
		/** The moving shine; off for many small foils at once */
		shiny?: boolean;
		rows?: number;
	}
	let {
		revealed,
		onreveal,
		onscratchstart,
		onscratch,
		onscratchend,
		label,
		cols = 1,
		rows = 1,
		tool = 'coin',
		shiny = true
	}: Props = $props();

	let wrap = $state<HTMLDivElement>();
	let canvas = $state<HTMLCanvasElement>();
	let dust = $state<HTMLCanvasElement>();
	let sheen = $state<HTMLCanvasElement>();
	let sheenCtx: CanvasRenderingContext2D | null = null;
	let ctx: CanvasRenderingContext2D | null = null;
	let dustCtx: CanvasRenderingContext2D | null = null;
	/** What has been scratched, from the strokes: the canvas itself is never read back */
	let coverage: Coverage | null = null;
	let width = 0;
	let height = 0;
	let dpr = 1;
	let fired = false;
	let touched = false;

	interface Flake {
		x: number;
		y: number;
		vx: number;
		vy: number;
		r: number;
		spin: number;
		angle: number;
		life: number;
		color: string;
	}
	let flakesAlive: Flake[] = [];
	let frame = 0;
	const FALL = 160; // px below the foil the flakes can fall

	onMount(() => {
		const box = wrap!.getBoundingClientRect();
		width = box.width;
		height = box.height;
		dpr = Math.min(window.devicePixelRatio || 1, 3);
		canvas!.width = Math.round(width * dpr);
		canvas!.height = Math.round(height * dpr);
		ctx = canvas!.getContext('2d');
		ctx?.scale(dpr, dpr);
		dust!.width = Math.round(width * dpr);
		dust!.height = Math.round((height + FALL) * dpr);
		dustCtx = dust!.getContext('2d');
		dustCtx?.scale(dpr, dpr);
		coverage = new Coverage(width, height);
		sheen!.width = canvas!.width;
		sheen!.height = canvas!.height;
		sheenCtx = sheen!.getContext('2d');
		sheenCtx?.scale(dpr, dpr);
		paint();
		// Listen from the start: where permission is needed (iOS) nothing comes
		// until the player allows it, asked on their first touch
		listenTilt();
		if (shiny) sheenFrame = requestAnimationFrame(shine);
		// The words use the display font: paint again once it is in, unless scratched already
		document.fonts?.load(`400 20px Bungee`).then(() => {
			if (!touched) paint();
		});
		return () => {
			cancelAnimationFrame(frame);
			cancelAnimationFrame(sheenFrame);
			removeEventListener('deviceorientation', onTilt);
		};
	});

	// ---------- the shine, following the tilt ----------

	let sheenFrame = 0;
	let shineAt: Offset = { x: 0, y: 0 };
	let tiltBase: { beta: number; gamma: number } | null = null;
	let input: { offset: Offset; at: number } | null = null;
	let sheenDirty = true;
	let sheenPrev = 0;
	let listening = false;

	const needsTiltPermission = () =>
		typeof DeviceOrientationEvent !== 'undefined' &&
		'requestPermission' in (DeviceOrientationEvent as object);

	function listenTilt() {
		if (listening || typeof DeviceOrientationEvent === 'undefined') return;
		listening = true;
		addEventListener('deviceorientation', onTilt);
	}

	function onTilt(e: DeviceOrientationEvent) {
		if (e.beta === null || e.gamma === null) return;
		const reading = { beta: e.beta, gamma: e.gamma };
		// Centred on how the phone is held when the first reading comes
		tiltBase ??= reading;
		input = { offset: tiltOffset(reading, tiltBase), at: performance.now() };
	}

	function hover(e: PointerEvent) {
		// A mouse moves the shine; a finger tilts the phone instead
		if (e.pointerType !== 'mouse' || !canvas) return;
		input = {
			offset: pointerOffset({ x: e.clientX, y: e.clientY }, canvas.getBoundingClientRect()),
			at: performance.now()
		};
	}

	function shine(t: number) {
		const dt = Math.min(64, sheenPrev ? t - sheenPrev : 16);
		sheenPrev = t;
		const still = matchMedia('(prefers-reduced-motion: reduce)').matches;
		const target =
			input && t - input.at < IDLE_AFTER ? input.offset : still ? { x: 0, y: 0 } : drift(t);
		const next = approach(shineAt, target, dt);
		if (sheenDirty || Math.abs(next.x - shineAt.x) + Math.abs(next.y - shineAt.y) > 0.002) {
			shineAt = next;
			drawSheen();
		}
		if (!revealed) sheenFrame = requestAnimationFrame(shine);
	}

	/** A bright band and a faint rainbow across the foil, kept to the foil that is left */
	function drawSheen() {
		const c = sheenCtx;
		if (!c || !canvas) return;
		sheenDirty = false;
		c.globalCompositeOperation = 'source-over';
		c.clearRect(0, 0, width, height);
		const cx = width * (0.5 + shineAt.x * 0.65);
		const cy = height * (0.5 + shineAt.y * 0.65);
		const band = (spread: number, stops: [number, string][]) => {
			const g = c.createLinearGradient(cx - spread, cy - spread, cx + spread, cy + spread);
			for (const [at, color] of stops) g.addColorStop(at, color);
			c.fillStyle = g;
			c.fillRect(0, 0, width, height);
		};
		band(width * 0.9, [
			[0, 'rgba(255, 79, 139, 0)'],
			[0.3, 'rgba(255, 79, 139, 0.16)'],
			[0.42, 'rgba(255, 214, 107, 0.16)'],
			[0.55, 'rgba(52, 224, 161, 0.14)'],
			[0.68, 'rgba(87, 184, 255, 0.16)'],
			[1, 'rgba(87, 184, 255, 0)']
		]);
		band(width * 0.45, [
			[0, 'rgba(255, 255, 255, 0)'],
			[0.44, 'rgba(255, 255, 255, 0)'],
			[0.5, 'rgba(255, 255, 255, 0.6)'],
			[0.56, 'rgba(255, 255, 255, 0)'],
			[1, 'rgba(255, 255, 255, 0)']
		]);
		// Only where foil is left: the scratched canvas cuts it out (drawn, never read back)
		c.globalCompositeOperation = 'destination-in';
		c.drawImage(canvas, 0, 0, width, height);
	}

	/** The foil: brushed silver, a sheen, and the words to scratch */
	function paint() {
		if (!ctx) return;
		ctx.globalCompositeOperation = 'source-over';
		const g = ctx.createLinearGradient(0, 0, width, height);
		// The theme's foil colours, from its CSS tokens
		const css = getComputedStyle(wrap!);
		const token = (name: string, fallback: string) => css.getPropertyValue(name).trim() || fallback;
		g.addColorStop(0, token('--foil-1', '#cfcbe0'));
		g.addColorStop(0.45, token('--foil-2', '#a9a3c4'));
		g.addColorStop(0.55, token('--foil-3', '#e9e6f5'));
		g.addColorStop(1, token('--foil-4', '#8f89ad'));
		ctx.fillStyle = g;
		ctx.fillRect(0, 0, width, height);
		// Brushed grain
		for (let i = 0; i < width * height * 0.02; i++) {
			ctx.fillStyle = Math.random() < 0.5 ? 'rgba(255,255,255,0.18)' : 'rgba(40,30,80,0.08)';
			ctx.fillRect(Math.random() * width, Math.random() * height, 1.5, 1.5);
		}
		// Words repeated on a slant
		ctx.save();
		ctx.translate(width / 2, height / 2);
		ctx.rotate(-0.22);
		ctx.font = `400 ${Math.round(width / 12)}px Bungee, 'Arial Black', sans-serif`;
		ctx.textAlign = 'center';
		ctx.textBaseline = 'middle';
		ctx.fillStyle = token('--foil-ink', 'rgba(70, 55, 120, 0.22)');
		const word = ctx.measureText(label).width + width / 8;
		const row = width / 5;
		for (let y = -height; y <= height; y += row) {
			const shift = (Math.abs(Math.round(y / row)) % 2) * (word / 2);
			for (let x = -width - word; x <= width + word; x += word) {
				ctx.fillText(label, x + shift, y);
			}
		}
		ctx.restore();
		// A frame so the foil reads as a sticker
		ctx.strokeStyle = 'rgba(255,255,255,0.5)';
		ctx.lineWidth = 2;
		ctx.strokeRect(4, 4, width - 8, height - 8);
	}

	let last: { x: number; y: number; t: number } | null = null;
	let pending = false;
	/** Where the tool shows, while the finger is down */
	let pen = $state<{ x: number; y: number } | null>(null);
	let still: ReturnType<typeof setTimeout> | undefined;

	function point(e: PointerEvent) {
		const box = canvas!.getBoundingClientRect();
		return { x: e.clientX - box.left, y: e.clientY - box.top, t: e.timeStamp };
	}

	function scrape(from: { x: number; y: number }, to: { x: number; y: number }, size: number) {
		coverage?.stroke(from.x, from.y, to.x, to.y, size);
		for (const c of [ctx]) {
			if (!c) continue;
			c.globalCompositeOperation = 'destination-out';
			c.lineCap = 'round';
			c.lineJoin = 'round';
			c.lineWidth = size;
			c.beginPath();
			c.moveTo(from.x, from.y);
			// A zero-length line draws nothing: nudge it so a tap leaves a dot
			c.lineTo(to.x + (from.x === to.x && from.y === to.y ? 0.1 : 0), to.y);
			c.stroke();
		}
	}

	function down(e: PointerEvent) {
		if (revealed) return;
		e.preventDefault();
		onscratchstart?.();
		// Inside the touch: the one moment iOS lets the page ask for the tilt
		if (needsTiltPermission()) askTilt().then((ok) => ok && listenTilt());
		canvas!.setPointerCapture(e.pointerId);
		touched = true;
		last = point(e);
		pen = { x: last.x, y: last.y };
		scrape(last, last, brush(0, TOOLS[tool].brush));
	}

	function move(e: PointerEvent) {
		hover(e);
		if (!last || revealed) return;
		const events = e.getCoalescedEvents?.() ?? [e];
		let speed = 0;
		for (const ev of events.length ? events : [e]) {
			const p = point(ev);
			const dist = Math.hypot(p.x - last.x, p.y - last.y);
			speed = dist / Math.max(1, p.t - last.t);
			scrape(last, p, brush(speed, TOOLS[tool].brush));
			shed(p, dist, p.x - last.x);
			last = p;
		}
		if (last) pen = { x: last.x, y: last.y };
		onscratch?.(speed);
		sheenDirty = true;
		// A finger resting on the foil makes no sound: no move for a moment is a stop
		clearTimeout(still);
		still = setTimeout(() => onscratch?.(0), 90);
		measureSoon();
	}

	function up() {
		clearTimeout(still);
		if (last) onscratchend?.();
		last = null;
		pen = null;
		measure();
	}

	function measureSoon() {
		if (pending) return;
		pending = true;
		setTimeout(() => {
			pending = false;
			measure();
		}, 120);
	}

	function measure() {
		if (!coverage || fired) return;
		if (ready(coverage.shares(cols, rows))) {
			fired = true;
			onreveal();
		}
	}

	/** Foil flakes coming off where the finger goes */
	function shed(at: { x: number; y: number }, distance: number, dx: number) {
		if (matchMedia('(prefers-reduced-motion: reduce)').matches) return;
		for (let i = 0; i < flakes(distance); i++) {
			flakesAlive.push({
				x: at.x + (Math.random() - 0.5) * 16,
				y: at.y + (Math.random() - 0.5) * 16,
				vx: dx * 0.02 + (Math.random() - 0.5) * 0.12,
				vy: -Math.random() * 0.15,
				r: 1.2 + Math.random() * 2.6,
				spin: (Math.random() - 0.5) * 0.02,
				angle: Math.random() * Math.PI,
				life: 1,
				color: TOOLS[tool].flakes[Math.floor(Math.random() * TOOLS[tool].flakes.length)]
			});
		}
		if (flakesAlive.length > 400) flakesAlive.splice(0, flakesAlive.length - 400);
		if (!frame) frame = requestAnimationFrame(tick);
	}

	let prev = 0;
	function tick(t: number) {
		const dt = Math.min(32, prev ? t - prev : 16);
		prev = t;
		if (!dustCtx) return;
		dustCtx.clearRect(0, 0, width, height + FALL);
		for (const f of flakesAlive) {
			f.vy += 0.0009 * dt;
			f.x += f.vx * dt;
			f.y += f.vy * dt;
			f.angle += f.spin * dt;
			f.life -= dt / 1100;
			dustCtx.save();
			dustCtx.globalAlpha = Math.max(0, f.life);
			dustCtx.translate(f.x, f.y);
			dustCtx.rotate(f.angle);
			dustCtx.fillStyle = f.color;
			const glow = TOOLS[tool].glow;
			if (glow) {
				dustCtx.shadowColor = glow;
				dustCtx.shadowBlur = 8;
			}
			dustCtx.fillRect(-f.r, -f.r / 2, f.r * 2, f.r);
			dustCtx.restore();
		}
		flakesAlive = flakesAlive.filter((f) => f.life > 0 && f.y < height + FALL);
		if (flakesAlive.length) {
			frame = requestAnimationFrame(tick);
		} else {
			frame = 0;
			prev = 0;
			dustCtx.clearRect(0, 0, width, height + FALL);
		}
	}
</script>

<div class="foil" class:revealed bind:this={wrap}>
	<canvas
		bind:this={canvas}
		class="scratch"
		aria-label={label}
		onpointerdown={down}
		onpointermove={move}
		onpointerup={up}
		onpointercancel={up}
	></canvas>
	<canvas bind:this={sheen} class="sheen" aria-hidden="true"></canvas>
	<canvas bind:this={dust} class="dust" aria-hidden="true" style:--fall="{FALL}px"></canvas>
	{#if pen && !revealed}
		<!-- The tool in hand, just above the finger so the finger does not hide it -->
		<span class="pen" aria-hidden="true" style:left="{pen.x}px" style:top="{pen.y}px"
			><ToolIcon {tool} /></span
		>
	{/if}
</div>

<style>
	.foil {
		position: absolute;
		inset: 0;
	}
	.scratch {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		border-radius: 14px;
		/* The finger scratches, the page does not scroll */
		touch-action: none;
		cursor: grab;
		transition:
			opacity 0.45s ease,
			transform 0.45s ease;
	}
	.sheen {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		pointer-events: none;
		transition: opacity 0.45s ease;
	}
	.revealed .sheen {
		opacity: 0;
	}
	.revealed .scratch {
		opacity: 0;
		transform: scale(1.04);
		pointer-events: none;
	}
	.pen {
		position: absolute;
		z-index: 2;
		width: 44px;
		height: 44px;
		pointer-events: none;
		transform: translate(-30%, -95%) rotate(-20deg);
		filter: drop-shadow(0 4px 6px rgba(0, 0, 0, 0.4));
	}
	.dust {
		position: absolute;
		left: 0;
		top: 0;
		width: 100%;
		height: calc(100% + var(--fall));
		pointer-events: none;
		z-index: 1;
	}
</style>
