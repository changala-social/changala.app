import { useEffect, useRef } from 'react';
import * as d3 from 'd3';
import { useNavigate } from 'react-router-dom';
import type { GetNodeGraphResponse } from '../../generated/types';

interface GraphViewProps {
  data: GetNodeGraphResponse;
  centerUri: string;
  width?: number;
  height?: number;
}

interface SimNode extends d3.SimulationNodeDatum {
  uri: string;
  title: string;
  authorDid: string;
  tags: string[];
  summary?: string;
}

interface SimLink extends d3.SimulationLinkDatum<SimNode> {
  label?: string;
}

export function GraphView({ data, centerUri, width = 800, height = 600 }: GraphViewProps) {
  const svgRef = useRef<SVGSVGElement>(null);
  const navigate = useNavigate();

  useEffect(() => {
    if (!svgRef.current || !data.nodes.length) return;

    const svg = d3.select(svgRef.current);
    svg.selectAll('*').remove();

    const nodes: SimNode[] = data.nodes.map((n) => ({ ...n }));
    const nodeMap = new Map(nodes.map((n) => [n.uri, n]));
    const links: SimLink[] = data.edges
      .filter((e) => nodeMap.has(e.fromUri) && nodeMap.has(e.toUri))
      .map((e) => ({
        source: e.fromUri,
        target: e.toUri,
        label: e.label,
      }));

    const g = svg.append('g');

    // Zoom
    const zoom = d3.zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.2, 4])
      .on('zoom', (event) => g.attr('transform', event.transform));
    svg.call(zoom);

    const simulation = d3.forceSimulation(nodes)
      .force('link', d3.forceLink<SimNode, SimLink>(links).id((d) => d.uri).distance(100))
      .force('charge', d3.forceManyBody().strength(-300))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(30));

    // Links
    const link = g.append('g')
      .selectAll('line')
      .data(links)
      .enter().append('line')
      .attr('stroke', '#94a3b8')
      .attr('stroke-width', 1.5)
      .attr('stroke-opacity', 0.6);

    // Link labels
    const linkLabel = g.append('g')
      .selectAll('text')
      .data(links.filter((l) => l.label))
      .enter().append('text')
      .text((d) => d.label || '')
      .attr('font-size', '10px')
      .attr('fill', '#94a3b8')
      .attr('text-anchor', 'middle')
      .attr('opacity', 0);

    // Nodes
    const node = g.append('g')
      .selectAll<SVGCircleElement, SimNode>('circle')
      .data(nodes)
      .enter().append('circle')
      .attr('r', (d) => d.uri === centerUri ? 12 : 8)
      .attr('fill', (d) => d.uri === centerUri ? '#7c3aed' : '#2563eb')
      .attr('stroke', '#fff')
      .attr('stroke-width', 2)
      .attr('cursor', 'pointer')
      .on('click', (_, d) => navigate(`/brain/${encodeURIComponent(d.uri)}`))
      .on('mouseover', function () { d3.select(this).attr('r', 14); })
      .on('mouseout', function (_, d) { d3.select(this).attr('r', d.uri === centerUri ? 12 : 8); })
      .call(d3.drag<SVGCircleElement, SimNode>()
        .on('start', (event, d) => { if (!event.active) simulation.alphaTarget(0.3).restart(); d.fx = d.x; d.fy = d.y; })
        .on('drag', (event, d) => { d.fx = event.x; d.fy = event.y; })
        .on('end', (event, d) => { if (!event.active) simulation.alphaTarget(0); d.fx = null; d.fy = null; })
      );

    // Node labels
    const nodeLabel = g.append('g')
      .selectAll('text')
      .data(nodes)
      .enter().append('text')
      .text((d) => d.title.length > 20 ? d.title.slice(0, 18) + '...' : d.title)
      .attr('font-size', '11px')
      .attr('fill', '#e2e8f0')
      .attr('text-anchor', 'middle')
      .attr('dy', -16);

    // Tooltip
    const tooltip = d3.select('body').append('div')
      .attr('class', 'fixed bg-gray-900 text-white text-xs rounded px-2 py-1 pointer-events-none z-50')
      .style('opacity', 0);

    node.on('mouseover', (event, d) => {
      tooltip.style('opacity', 1).html(d.summary || d.title)
        .style('left', event.pageX + 10 + 'px').style('top', event.pageY - 10 + 'px');
    }).on('mouseout', () => tooltip.style('opacity', 0));

    // Edge label on hover
    link.on('mouseover', (_, d) => {
      const idx = links.indexOf(d);
      linkLabel.filter((_, j) => j === idx).attr('opacity', 1);
    }).on('mouseout', () => linkLabel.attr('opacity', 0));

    simulation.on('tick', () => {
      link
        .attr('x1', (d: any) => d.source.x).attr('y1', (d: any) => d.source.y)
        .attr('x2', (d: any) => d.target.x).attr('y2', (d: any) => d.target.y);
      node.attr('cx', (d) => d.x!).attr('cy', (d) => d.y!);
      nodeLabel.attr('x', (d) => d.x!).attr('y', (d) => d.y!);
      linkLabel
        .attr('x', (d: any) => ((d.source.x + d.target.x) / 2))
        .attr('y', (d: any) => ((d.source.y + d.target.y) / 2));
    });

    return () => {
      simulation.stop();
      tooltip.remove();
    };
  }, [data, centerUri, width, height, navigate]);

  return (
    <svg
      ref={svgRef}
      width={width}
      height={height}
      className="w-full h-full bg-surface rounded-lg border border-border"
      viewBox={`0 0 ${width} ${height}`}
    />
  );
}
