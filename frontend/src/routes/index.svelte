<script lang="ts" context="module">
    import type { Load } from "@sveltejs/kit";
    import { variables } from "$lib/variables";
    import Sidebar from "$lib/Sidebar.svelte";

    export const load: Load = async ({ fetch }) => {
        const res = await fetch(
            variables.apiUrl +
                "/posts?title=any&published=true&limit=5&published_date=any"
        );

        if (!res.ok) {
            const data = await res.json();
            const message = data["message"];

            const error = new Error(message);
            return { status: res.status, error };
        } else {
            const data = await res.json();

            return { props: { posts: data["data"] } };
        }
    };
</script>

<script lang="ts">
    import type { Post } from "$lib/types";

    export let posts: Post[];
</script>

<div class="main-container">
    <div class="feed">
        {#each posts as post}
            <div class="post-item">
                <a href={`/posts/${post.id}`} class="post-title">{post.title}</a
                >
                <p class="post-date">{post.published_date}</p>
                <p class="post-summary">{post.summary}</p>
            </div>
        {/each}
    </div>
    <Sidebar />
</div>
