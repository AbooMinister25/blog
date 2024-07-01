import MiniSearch from "minisearch";

function debounce(func, wait) {
    var timeout;

    return function () {
        var context = this;
        var args = arguments;
        clearTimeout(timeout);

        timeout = setTimeout(function () {
            timeout = null;
            func.apply(context, args);
        }, wait);
    };
}

function initSearch() {
    let searchInput = document.getElementById("search");
    let searchResults = document.querySelector(".search-results");
    let searchResultsItems = document.querySelector(".search-results-items");

    var minisearch;

    var initIndex = async function () {
        if (minisearch === undefined) {
            minisearch = await fetch("/index.json").then(async function (
                response
            ) {
                let json = await response.json();
                inc = 0;
                json.forEach((item) => {
                    item.id = inc++;
                });

                var ms = new MiniSearch({
                    fields: ["title", "body", "tags"],
                    storeFields: [
                        "title",
                        "summary",
                        "date",
                        "tags",
                        "permalink",
                    ],
                });
                ms.addAll(json);

                return ms;
            });
        }

        let res = await minisearch;
        return res;
    };

    var currentTerm;

    const options = {
        year: "numeric",
        month: "short",
        day: "numeric",
    };

    searchInput.addEventListener("input", async function () {
        let term = searchInput.value.trim();
        if (term === currentTerm) {
            return;
        }

        searchResults.style.display = term === "" ? "none" : "block";
        searchResultsItems.innerHTML = "";
        currentTerm = term;

        if (term === "") {
            return;
        }

        let results = (await initIndex()).search(term, {
            boost: { title: 2 },
            fuzzy: 0.2,
            prefix: true,
            combineWith: "AND",
        });
        console.log(results);
        if (results.length == 0) {
            searchResults.style.display = "none";
            return;
        }

        for (const result of results) {
            let date = new Date(result.date);
            let item = document.createElement("div");
            item.innerHTML = `
            <h1 class="post-header">
                <a href="${result.permalink}">${result.title}</a>
            </h1>
            <div>
            ${result.summary}
            </div>
            <p class="post-details">
                ${date.toLocaleDateString("en-US", options)} *
                ${result.tags.join(", ")}
            </p>
            `;
            searchResultsItems.appendChild(item);
        }
    });
}

document.addEventListener("DOMContentLoaded", initSearch);
