/** Shapes of the API's responses, see backend/src/routes */

export type Series = 'lucky_three' | 'number_match' | 'ticket_chain' | 'cash_wheel' | 'golden';
export type Source = 'pocket' | 'envelope' | 'prize' | 'shop' | 'reward';

export type ThemeId = 'midnight' | 'sunset' | 'mint' | 'neon' | 'royal';
export type ToolId = 'coin' | 'key' | 'eraser' | 'laser';
export type Cosmetic = { kind: 'theme'; id: ThemeId } | { kind: 'tool'; id: ToolId };

export interface TicketDto {
	id: number;
	series: Series;
	source: Source;
	created_at: string;
}

export interface Pocket {
	count: number;
	size: number;
	/** When the next ticket comes, null while full */
	next_at: string | null;
}

export interface Contents {
	coins: number;
	tickets: Series[];
}

export interface EnvelopeDto {
	available: boolean;
	streak: number;
	best: number;
	day: number;
	contents: Contents;
	next_at: string;
}

export interface MeDto {
	username: string;
	coins: number;
	pocket: Pocket;
	envelope: EnvelopeDto;
	theme: ThemeId;
	tool: ToolId;
	/** Cosmetics owned besides the defaults */
	owned: Cosmetic[];
	level: LevelDto;
	tickets: TicketDto[];
}

export interface LevelDto {
	level: number;
	xp: number;
	/** Where this level started and the next begins */
	from: number;
	to: number;
}

export interface Reward {
	/** A level reached, a quest key or an achievement id */
	what: string;
	reward: Prize;
}

/** What an action's progress gave */
export interface Progress {
	xp_gained: number;
	level: LevelDto;
	level_ups: Reward[];
	quests: Reward[];
	bonus: Prize | null;
	achievements: Reward[];
	tickets: TicketDto[];
	coins: number;
}

export interface QuestDto {
	key: string;
	goal: number;
	progress: number;
	done: boolean;
	reward: Prize;
}

export interface QuestsDto {
	items: QuestDto[];
	bonus: Prize;
	bonus_done: boolean;
	rerolled: boolean;
	resets_at: string;
}

export interface AchievementDto {
	id: string;
	reward: Prize;
	earned_at: string | null;
	progress: [number, number] | null;
}

export type Prize =
	| { kind: 'nothing' }
	| { kind: 'coins'; amount: number }
	| { kind: 'ticket'; series: Series }
	| { kind: 'tickets'; series: Series[] }
	| { kind: 'pocket_refill' }
	| { kind: 'cosmetic'; item: Cosmetic };

export type LuckySymbol = 'gem' | 'coin' | 'clover' | 'ticket' | 'hourglass' | 'star';

export interface LuckyLayout {
	cells: LuckySymbol[];
	win: LuckySymbol | null;
}

export interface NumberLayout {
	winning: number[];
	spots: { n: number; coins: number; hit: boolean }[];
}

export interface ChainLayout {
	panels: (Series | null)[];
}

export interface WheelLayout {
	base: number;
	wheel: number[];
	/** The segment the wheel stops on */
	stop: number;
}

export interface GoldenLayout {
	prize: Prize;
}

export interface Scratched {
	id: number;
	series: Series;
	layout: unknown;
	prize: Prize;
	coins: number;
	/** Tickets won */
	tickets: TicketDto[];
	/** The cosmetic won was already owned: coins instead */
	duplicate: boolean;
	progress: Progress;
}

export interface ShopDto {
	tickets: { series: Series; price: number; level: number }[];
	cosmetics: { item: Cosmetic; price: number }[];
}

export interface Bought {
	coins: number;
	ticket: TicketDto | null;
	item: Cosmetic | null;
	progress: Progress;
}

export interface PocketClaim {
	tickets: TicketDto[];
	pocket: Pocket;
	progress: Progress;
}

export interface EnvelopeClaim {
	day: number;
	contents: Contents;
	coins: number;
	tickets: TicketDto[];
	progress: Progress;
}
