function reverse(input) {
  return input.split("").reverse().join("");
}

function wordCount(input) {
  return input.trim().split(/\s+/).length;
}

module.exports = { reverse, wordCount };
