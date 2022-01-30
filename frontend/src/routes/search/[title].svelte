<script lang="ts" context="module">
    import type { Load } from "@sveltejs/kit";
    import { variables } from "$lib/variables";

    export const load: Load = async ({ page: { params }, fetch }) => {
        const res = await fetch(
            variables.apiUrl +
                `/posts?title=${params["title"]}&published=true&limit=5&published_date=any`
        );

        if (res.ok) {
            const data = await res.json();
            return { props: { posts: data["data"], status: "success" } };
        } else if (res.status == 404) {
            return {
                props: { posts: [], status: "not-found" },
            };
        } else {
            const data = await res.json();
            const message = data["message"];

            const error = new Error(message);
            return { status: res.status, error };
        }
    };
</script>

<script lang="ts">
    import type { Post } from "$lib/types";

    export let posts: Post[];
    export let status: string;
</script>

<div class="search-results">
    <div>
        {#if status === "success"}
            <h1 class="results-header">Results</h1>
            <div class="search-feed">
                {#each posts as post}
                    <div class="post-item">
                        <a href={`/posts/${post.id}`} class="post-title"
                            >{post.title}</a
                        >
                        <p class="post-date">{post.published_date}</p>
                        <p class="post-summary">{post.summary}</p>
                    </div>
                {/each}
            </div>
        {:else if status === "not-found"}
            <h1 class="results-header">No Results Found</h1>
        {/if}
    </div>
</div>
