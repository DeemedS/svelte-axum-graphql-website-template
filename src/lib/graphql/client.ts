// src/lib/graphql/client.ts
export const GRAPHQL_ENDPOINT =
    import.meta.env.VITE_GRAPHQL_ENDPOINT || 'http://localhost:3000/';

export async function gql<T>(
    query: string,
    variables?: Record<string, any>
): Promise<T> {
    const resp = await fetch(GRAPHQL_ENDPOINT, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ query, variables }),
    });

    if (!resp.ok) {
        throw new Error(`HTTP ${resp.status}: ${await resp.text()}`);
    }

    const json = await resp.json();
    if (json.errors) {
        throw new Error(json.errors.map((e: any) => e.message).join('; '));
    }
    return json.data;
}
