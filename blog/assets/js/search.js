import MiniSearch from "minisearch";
import { PorterStemmer } from "porter-stemmer-js";

function extractTextNodes(content) {
    var content_el = document.createElement("div");
    content_el.innerHTML = content;

    var walker = document.createTreeWalker(content_el, NodeFilter.SHOW_TEXT);

    var node;
    var textNodes = [];

    while ((node = walker.nextNode())) {
        textNodes.push(node.nodeValue);
    }

    var text = new Array();
    for (var t of textNodes) {
        text.push(t);
    }

    return text.join(" ");
}

// Taken from the zola website, which in turn took it from mdbook lol.
function makeTeaser(body, terms) {
    var TERM_WEIGHT = 40;
    var NORMAL_WORD_WEIGHT = 2;
    var FIRST_WORD_WEIGHT = 8;
    var TEASER_MAX_WORDS = 30;

    var stemmedTerms = terms.map(function (w) {
        return PorterStemmer(w.toLowerCase());
    });
    var termFound = false;
    var index = 0;
    var weighted = []; // contains elements of ["word", weight, index_in_document]

    // split in sentences, then words
    var sentences = body.toLowerCase().split(". ");

    for (var i in sentences) {
        var words = sentences[i].split(" ");
        var value = FIRST_WORD_WEIGHT;

        for (var j in words) {
            var word = words[j];

            if (word.length > 0) {
                for (var k in stemmedTerms) {
                    if (PorterStemmer(word).startsWith(stemmedTerms[k])) {
                        value = TERM_WEIGHT;
                        termFound = true;
                    }
                }
                weighted.push([word, value, index]);
                value = NORMAL_WORD_WEIGHT;
            }

            index += word.length;
            index += 1; // ' ' or '.' if last word in sentence
        }

        index += 1; // because we split at a two-char boundary '. '
    }

    if (weighted.length === 0) {
        return body;
    }

    var windowWeights = [];
    var windowSize = Math.min(weighted.length, TEASER_MAX_WORDS);
    // We add a window with all the weights first
    var curSum = 0;
    for (var i = 0; i < windowSize; i++) {
        curSum += weighted[i][1];
    }
    windowWeights.push(curSum);

    for (var i = 0; i < weighted.length - windowSize; i++) {
        curSum -= weighted[i][1];
        curSum += weighted[i + windowSize][1];
        windowWeights.push(curSum);
    }

    // If we didn't find the term, just pick the first window
    var maxSumIndex = 0;
    if (termFound) {
        var maxFound = 0;
        // backwards
        for (var i = windowWeights.length - 1; i >= 0; i--) {
            if (windowWeights[i] > maxFound) {
                maxFound = windowWeights[i];
                maxSumIndex = i;
            }
        }
    }

    var teaser = [];
    var startIndex = weighted[maxSumIndex][2];
    for (var i = maxSumIndex; i < maxSumIndex + windowSize; i++) {
        var word = weighted[i];
        if (startIndex < word[2]) {
            // missing text from index to start of `word`
            teaser.push(body.substring(startIndex, word[2]));
            startIndex = word[2];
        }

        // add <em/> around search terms
        if (word[1] === TERM_WEIGHT) {
            teaser.push("<b>");
        }
        startIndex = word[2] + word[0].length;
        teaser.push(body.substring(word[2], startIndex));

        if (word[1] === TERM_WEIGHT) {
            teaser.push("</b>");
        }
    }

    return teaser.join("");
}

function initSearch() {
    let searchInput = document.getElementById("search");
    let searchResults = document.querySelector(".post-list");
    let searchResultsItems = document.querySelector(".search-results-items");

    searchResults.style.display = "none";

    var minisearch;

    var initIndex = async function () {
        if (minisearch === undefined) {
            minisearch = await fetch("/index.json").then(async function (
                response
            ) {
                let json = await response.json();

                var ms = new MiniSearch({
                    idField: "hash",
                    fields: [
                        "document.frontmatter.title",
                        "raw_content",
                        "document.frontmatter.tags",
                    ],
                    storeFields: [
                        "document.frontmatter.title",
                        "document.frontmatter.date",
                        "document.frontmatter.tags",
                        "document.summary",
                        "permalink",
                        "document.content",
                    ],
                    searchOptions: {
                        boost: { title: 2 },
                        fuzzy: 0.2,
                        prefix: true,
                        combineWith: "AND",
                    },
                    extractField: (document, fieldName) => {
                        return fieldName
                            .split(".")
                            .reduce((doc, key) => doc && doc[key], document);
                    },
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

        let results = (await initIndex()).search(term);
        if (results.length == 0) {
            searchResults.style.display = "none";
            return;
        }

        for (const result of results) {
            let text = extractTextNodes(result["document.content"]);
            let teaser = makeTeaser(text, result.terms);
            let tags_teaser = makeTeaser(
                result["document.frontmatter.tags"].join(", "),
                result.terms
            );

            let date = new Date(result["document.frontmatter.date"]);
            let item = document.createElement("div");
            item.innerHTML = `
            <h1 class="post-header">
                <a href="${result.permalink}">${
                result["document.frontmatter.title"]
            }</a>
            </h1>
            <div>
            ${
                teaser.includes("<b>")
                    ? teaser + "..."
                    : result["document.summary"]
            }
            </div>
            <p class="post-details" style="b { font-size: 5rem; }">
                ${date.toLocaleDateString("en-US", options)} *
                ${tags_teaser}
            </p>
            `;
            searchResultsItems.appendChild(item);
        }
    });
}

document.addEventListener("DOMContentLoaded", initSearch);
