<script lang="ts" context="module">
    import type { Load } from "@sveltejs/kit";
    import { variables } from "$lib/variables";

    export const load: Load = async ({ fetch }) => {
        const res = await fetch(
            variables.apiUrl +
                "/posts?title=any&published=true&limit=10&published_date=any"
        );
        const data = await res.json();

        return { props: { posts: data["data"] } };
    };
</script>

<script lang="ts">
    import type { Post } from "$lib/types";

    export let posts: Post[];
</script>

<body>
    <div class="flex">
        <div
            class="flex justify-center flex-col items-center divide-y divide-oneblack/60"
        >
            {#each posts as post}
                <div class="w-screen lg:max-w-3xl py-10 overflow-hidden">
                    <div class="px-6 py-4 shadow-lg rounded hover:shadow-2xl border-2 border-grey7">
                        <a
                            href={`/posts/${post.id}`}
                            class="font-bold text-3xl text-grey3 hover:opacity-80"
                        >
                            {post.title}
                        </a>
                        <p
                            class="text-grey2 opacity-70 font-medium text-sm mb-10 mt-2"
                        >
                            {post.published_date}
                        </p>
                        <p class="mb-10 text-oneblack opacity-80">
                            {post.summary}
                        </p>
                    </div>
                </div>
            {/each}
        </div>
    </div>
</body>
