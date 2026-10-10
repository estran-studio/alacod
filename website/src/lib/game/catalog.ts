export const games = [
	{
		id: 'zombies',
		name: 'Zombies',
		milestone: 'M0',
		genre: 'Survie · Coopération',
		description: 'Tenir une vague de plus. Ouvrir des passages, acheter ses armes et relever ses coéquipiers.',
		features: ['Vagues de zombies', 'Achats et améliorations', 'Réanimation en coopération'],
		accent: 'ember',
		image: '/images/zombies.png'
	},
	{
		id: 'throne',
		name: 'Throne',
		milestone: 'M1',
		genre: 'Roguelike · Exploration',
		description: 'Traverser des cavernes, changer d’arsenal et choisir les mutations qui feront la prochaine partie.',
		features: ['Cavernes et étages', 'Armes et projectiles composables', 'Progression et mutations'],
		accent: 'sage',
		image: ''
	}
] as const;

export type GameId = (typeof games)[number]['id'];
export type Release = { buildId: string; manifest: string; solo: boolean; online: boolean };
export type Releases = { schemaVersion: 1; games: Partial<Record<GameId, Release>> };
export type BuildManifest = {
	schemaVersion: 1;
	gameId: GameId;
	buildId: string;
	engineCommit: string;
	module: string;
	wasm: string;
	assets: string;
	generatedAt: string;
};

export function findGame(id: string | null) {
	return games.find((game) => game.id === (id === 'map_explorer' ? 'zombies' : id));
}

export async function getReleases(fetcher: typeof fetch = fetch): Promise<Releases> {
	const response = await fetcher('/releases.json', { cache: 'no-store' });
	if (!response.ok) throw new Error('Le catalogue des versions est indisponible.');
	const data: Releases = await response.json();
	if (data.schemaVersion !== 1 || !data.games) throw new Error('Catalogue de versions invalide.');
	return data;
}
