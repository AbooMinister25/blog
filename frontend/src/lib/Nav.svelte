<script lang="ts">
    import { fade } from "svelte/transition";

    let menuOpen = false;

    function toggleMenu() {
        menuOpen = !menuOpen;
    }
</script>

<nav class='border-grey6 border-b-2 bg-grey7'>
    <div class="mx-auto px-4 py-4">
        <div class="flex justify-between">
            <div class="flex space-x-7">
                <div>
                    <a
                        class="bg-oneblack p-2 flex-shrink-0 text-3xl text-grey7"
                        href="/">Rayyan</a
                    >
                </div>

                <div class="hidden md:flex items-center space-x-1">
                    <div class="relative mx-auto text-grey3">
                        <input
                            class="inline-flex bg-grey6 h-10 px-3 placeholder-grey3 focus:outline-none rounded w-96 focus:ring-2 focus:ring-oneblack"
                            type="search"
                            name="search"
                            placeholder="Explore the depths..."
                        />
                        <button
                            type="submit"
                            class="absolute right-0 mt-2 mr-2"
                        >
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                class="h-6 w-6"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke="currentColor"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                                />
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
            <div class="hidden md:flex items-center space-x-3">
                <a
                    href=""
                    class="py-1 px-2 font-semibold text-grey3 link-underline"
                >
                    All Posts
                </a>
                <a
                    href=""
                    class="py-1 px-2 font-semibold text-grey3 link-underline"
                >
                    Playlists
                </a>
            </div>

            <div class="md:hidden flex items-center">
                <button
                    class="ml-auto lg:hidden"
                    type="button"
                    on:click={toggleMenu}
                >
                    {#if !menuOpen}
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            class="h-12 w-12"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke="currentColor"
                            in:fade
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width={1}
                                d="M4 6h16M4 12h16M4 18h16"
                            />
                        </svg>
                    {:else}
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            class="h-12 w-12"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke="currentColor"
                            in:fade
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="1"
                                d="M6 18L18 6M6 6l12 12"
                            />
                        </svg>
                    {/if}
                </button>
            </div>
        </div>
    </div>
    {#if menuOpen}
        <div class="px-4 pb-4" transition:fade>
            <ul>
                <li>
                    <a
                        href=""
                        class="block py-1 my-4 px-2 font-semibold text-grey3 link-underline"
                    >
                        All Posts
                    </a>
                </li>
                <li>
                    <a
                        href=""
                        class="block py-1 my-4 px-2 font-semibold text-grey3 link-underline"
                    >
                        Playlists
                    </a>
                </li>
                <li>
                    <div class="flex flex-row relative mx-auto text-grey3">
                        <input
                            class="bg-grey7 h-10 px-3 placeholder-grey3 focus:outline-none rounded w-96 focus:ring-2 focus:ring-oneblack grow"
                            type="search"
                            name="search"
                            placeholder="Explore the depths..."
                        />
                        <button
                            type="submit"
                            class="absolute right-0 mt-2 mr-2"
                        >
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                class="h-6 w-6"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke="currentColor"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                                />
                            </svg>
                        </button>
                    </div>
                </li>
            </ul>
        </div>
    {/if}
</nav>
