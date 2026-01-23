const perRow = 6;
const container = document.querySelector('.novel-list');
container.querySelectorAll('.novel-card.ghost').forEach(n => n.remove());
const items = container.querySelectorAll('.novel-card:not(.ghost)');
const missing = (perRow - (items.length % perRow)) % perRow;
for(let i=0;i<missing;i++){
    const ghost = document.createElement('div');
    ghost.className = 'novel-card ghost';
    container.appendChild(ghost);
}